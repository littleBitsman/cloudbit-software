extern crate execute;
extern crate json;
extern crate tungstenite;
extern crate url;

use core::time;
use execute::Execute;
use json::object;
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
            LEDCommand::Red => write!(f, "red"),
            LEDCommand::Green => write!(f, "green"),
            LEDCommand::Blue => write!(f, "blue"),
            LEDCommand::Purple => write!(f, "purple"),
            LEDCommand::Violet => write!(f, "violet"),
            LEDCommand::Teal => write!(f, "teal"),
            LEDCommand::Yellow => write!(f, "yellow"),
            LEDCommand::White => write!(f, "white"),
            LEDCommand::Off => write!(f, "off"),
            LEDCommand::Clownbarf => write!(f, "clownbarf"),
            LEDCommand::Blink => write!(f, "blink"),
            LEDCommand::Hold => write!(f, "hold"),
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

    match cmd.output() {
        Ok(output) => {
            let lines = String::from_utf8_lossy(&output.stdout);
            println!("{:?}", lines);
            let line = lines.lines().nth(0);
            if let Some(line) = line {
                u8::from_str(line.trim()).unwrap()
            } else {
                eprintln!("failed to read input properly: {}", line.unwrap_or("no line there"));
                0
            }
        },
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
    set_led(LEDCommand::Teal);
    set_led(LEDCommand::Blink);

    let url = std::fs::read_to_string("/usr/local/lb/cloud_client/server_url").unwrap_or("ws://chiseled-private-cauliflower.glitch.me/".to_owned());

    loop {
        let result = catch_unwind(|| start(url.as_str()));
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
fn start(conf: &str) {
    let url = Url::from_str(conf).unwrap();

    println!(
        "Attempting to connect to {} ({})",
        url.to_string(),
        url.host_str().unwrap()
    );

    set_led(LEDCommand::Hold);

    let mut current_input: u8 = 0;
    let request = Request::get(url.as_str())
        // .header("MAC-Address", conf.mac_address.as_str())
        // .header("CB-Id", conf.cb_id.as_str())
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
