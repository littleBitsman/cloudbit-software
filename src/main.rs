extern crate execute;
extern crate json;
extern crate tungstenite;
extern crate url;

mod conf;

use conf::cloud_config::{self, CloudClientConfig};
use core::time;
use execute::Execute;
use json::object;
use std::fs::read_to_string;
use std::panic::catch_unwind;
use std::process::Command;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use tungstenite::connect;
use tungstenite::handshake::client::{generate_key, Request};
use tungstenite::Message;
use url::Url;

#[allow(dead_code)]
enum LEDCommand {
    Red,
    Green,
    Blue,
    Purple,
    Violet,
    Teal,
    Yellow,
    White,
    Off,
    Clownbarf,
    Blink,
    Hold,
}

impl std::fmt::Display for LEDCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LEDCommand::Red => write!(f, "RED"),
            LEDCommand::Green => write!(f, "GREEN"),
            LEDCommand::Blue => write!(f, "BLUE"),
            LEDCommand::Purple => write!(f, "PURPLE"),
            LEDCommand::Violet => write!(f, "VIOLET"),
            LEDCommand::Teal => write!(f, "TEAL"),
            LEDCommand::Yellow => write!(f, "YELLOW"),
            LEDCommand::White => write!(f, "WHITE"),
            LEDCommand::Off => write!(f, "OFF"),
            LEDCommand::Clownbarf => write!(f, "CLOWNBARF"),
            LEDCommand::Blink => write!(f, "BLINK"),
            LEDCommand::Hold => write!(f, "HOLD"),
        }
    }
}

// UTILS

fn set_led_raw(str: String) -> bool {
    let mut cmd = Command::new("/usr/local/lb/LEDcolor/bin/setColor");
    cmd.arg(str);
    cmd.execute_check_exit_status_code(0).is_ok()
}

/// set led wow
fn set_led(arg: LEDCommand) -> bool {
    set_led_raw(arg.to_string())
}

/// get input (0-255)
fn get_input() -> u8 {
    let mut cmd = Command::new("/usr/local/lb/ADC/bin/getADC");
    cmd.arg("-1");
    match cmd.execute_output() {
        Ok(output) => output.stdout[0],
        Err(_) => 0,
    }
}

/// set output (as 0x0000 - 0xFFFF)
fn set_output(value: u16) -> bool {
    let mut cmd = Command::new("/usr/local/lb/DAC/bin/setDAC");
    cmd.arg(format!("{:04x}", value));
    cmd.execute_check_exit_status_code(0).is_ok()
}

// MAIN LOOP
fn main() {
    set_led(LEDCommand::Blue);

    let opts =
        cloud_config::parse("/usr/local/lb/etc/cloud_client.conf").unwrap_or(CloudClientConfig {
            cloud_url: "ws://chiseled-private-cauliflower.glitch.me/".to_string(),
            mac_address: read_to_string("/var/lb/mac").unwrap_or("ERROR_READING_MAC".to_string()),
            cb_id: read_to_string("/var/lb/id").unwrap_or("ERROR_READING_ID".to_string()),
        });

    set_led(LEDCommand::Blink);

    loop {
        let result = catch_unwind(|| start(opts.clone()));
        match result {
            Ok(()) => println!("you closed the connection somehow why??"),
            Err(err) => {
                println!("error {:?}", err);
                set_led(LEDCommand::Red);
                set_led(LEDCommand::Blink);
                sleep(time::Duration::from_secs(2));
                set_led(LEDCommand::Teal);
            }
        }
    }
}

// MAIN SOCKET
fn start(conf: CloudClientConfig) {
    let url = Url::from_str(&conf.cloud_url).unwrap();

    println!(
        "Attempting to connect to {} ({})",
        url.to_string(),
        url.host_str().unwrap()
    );

    set_led(LEDCommand::Hold);

    let mut current_input: u8 = 0;
    let request = Request::get(&conf.cloud_url)
        .header("MAC-Address", conf.mac_address.as_str())
        .header("CB-Id", conf.cb_id.as_str())
        .header("User-Agent", "littleARCH cloudBit")
        .header("Host", url.host_str().unwrap())
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-Websocket-Version", "13")
        .header("Sec-Websocket-Key", generate_key().as_str())
        .body(());

    if request.is_err() {
        panic!("{}", request.unwrap_err())
    }

    let (client_raw, _) = connect(request.unwrap()).unwrap();
    let client = Arc::new(Mutex::new(client_raw));

    println!("Successfully connected");

    set_led(LEDCommand::Green);

    let receive_loop = {
        let client = Arc::clone(&client);
        thread::spawn(move || {
            // Receive loop
            loop {
                let mut client = client.lock().unwrap();
                let message = match client.read() {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("Receive Loop: {:?}", e);
                        let _ = client.send(Message::Close(None));
                        return;
                    }
                };
                match message {
                    Message::Close(a) => {
                        // Got a close message, so send a close message and return
                        let _ = client.send(Message::Close(a));
                    }
                    Message::Ping(data) => {
                        match client.send(Message::Pong(data)) {
                            // Send a pong in response
                            Ok(()) => (),
                            Err(e) => {
                                eprintln!("Receive Loop: {:?}", e);
                                return;
                            }
                        }
                    }
                    Message::Text(data) => {
                        println!("{}", data);
                        let r = json::parse(&data);
                        if !r.is_ok() {
                            return eprintln!("bad packet from server")
                        }
                        let parsed = r.unwrap();
                        if !parsed.is_object() {
                            return eprintln!("bad packet from server")
                        }
                        match parsed {
                            json::JsonValue::Object(obj) => {
                                let opcode = obj["opcode"].as_u16().unwrap_or(0);

                                match opcode {
                                    0x2 => {
                                        // OUTPUT
                                        let new = obj["data"]["value"]
                                            .as_u16()
                                            .expect("bad output packet from server");
                                        set_output(new);
                                    }
                                    0x3 => println!("received hello packet"),
                                    0xF0 => {
                                        // SET LED
                                        set_led_raw(
                                            obj["color"]
                                                .as_str()
                                                .expect("[dev] bad set led packet")
                                                .to_lowercase()
                                                .to_string(),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        eprintln!("unknown content")
                    }
                }
            }
        })
    };

    loop {
        let right_now = get_input();
        if right_now != current_input {
            current_input = right_now;
            let mut client = client.lock().unwrap();
            let success = client
                .send(Message::Text(json::stringify(object! {
                    opcode: 0x1,
                    data: object! {
                        value: current_input
                    }
                })))
                .is_ok();
            if !success {
                break;
            }
        }
    }

    client.lock().unwrap().send(Message::Close(None)).unwrap();
    
    println!("connection closed");

    receive_loop.join().unwrap_or_default();
}
