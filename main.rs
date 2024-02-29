extern crate execute;
extern crate json;
extern crate tungstenite;

use core::time;
use std::fs;
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};

use json::object;
use tungstenite::connect;
use tungstenite::Message;

use execute::Execute;
use std::process::Command;

const CONNECTION: &'static str = "ws://192.168.1.155:3000/";

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
            LEDCommand::Violet => write!(f, "purple"),
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

fn set_color(arg: LEDCommand) -> bool {
    let mut cmd = Command::new("/usr/local/lb/LEDcolor/bin/setColor");
    cmd.arg(arg.to_string());
    cmd.execute_check_exit_status_code(0).is_ok()
}

fn get_input() -> u8 {
    let mut cmd = Command::new("/usr/local/lb/ADC/bin/getADC");
    cmd.arg("-1");
    match cmd.execute_output() {
        Ok(output) => output.stdout[0],
        Err(_) => 0,
    }
}

fn set_output(value: u16) -> bool {
    let mut cmd = Command::new("/usr/local/lb/DAC/bin/setDAC");
    cmd.arg(format!("{:04x}", value));
    cmd.execute_check_exit_status_code(0).is_ok()
}

fn main() {
    let mac_address = fs::read_to_string("/var/lb/mac").unwrap_or("ERROR_READING_MAC".to_string());
    let cb_id = fs::read_to_string("/var/lb/id").unwrap_or("ERROR_READING_ID".to_string());

    set_color(LEDCommand::Green);
    set_color(LEDCommand::Blink);
    loop {
        let result = catch_unwind(|| start(mac_address.clone(), cb_id.clone()));
        match result {
            Ok(_) => {}
            Err(_) => {
                set_color(LEDCommand::Red);
                set_color(LEDCommand::Blink);
                sleep(time::Duration::from_secs(2));
                set_color(LEDCommand::Green);
            }
        }
    }
}

fn start(mac_address: String, cb_id: String) {
    println!("Attempting to connect to {}", CONNECTION);

    set_color(LEDCommand::Hold);

    let mut current_input: u8 = 0;

    let request = tungstenite::handshake::client::Request::get(CONNECTION)
        .header("MAC-Address", mac_address)
        .header("CB-Id", cb_id)
        .header("User-Agent", "littleARCH cloudBit")
        .body(())
        .unwrap();

    let (client, _) = connect(request).unwrap();
    let client = Arc::new(Mutex::new(client));

    println!("Successfully connected");

    let receive_loop = {
        let client = Arc::clone(&client);
        thread::spawn(move || {
            // Receive loop
            loop {
                let mut client = client.lock().unwrap();
                let message = match client.read() {
                    Ok(m) => m,
                    Err(e) => {
                        println!("Receive Loop: {:?}", e);
                        let _ = client.send(Message::Close(None));
                        return;
                    }
                };
                match message {
                    Message::Close(a) => {
                        // Got a close message, so send a close message and return
                        let _ = client.send(Message::Close(a));
                        return;
                    }
                    Message::Ping(data) => {
                        match client.send(Message::Pong(data)) {
                            // Send a pong in response
                            Ok(()) => (),
                            Err(e) => {
                                println!("Receive Loop: {:?}", e);
                                return;
                            }
                        }
                    }
                    Message::Text(data) => {
                        println!("{}", data);
                        let r = json::parse(&data);
                        if !r.is_ok() {
                            return;
                        }
                        let parsed = r.unwrap();
                        if !parsed.is_object() {
                            return;
                        }

                        match parsed {
                            json::JsonValue::Object(obj) => {
                                if obj["opcode"] == 0x2 {
                                    // OUTPUT
                                    let new = obj["data"]["value"]
                                        .as_u16()
                                        .expect("bad output packet from server");
                                    set_output(new);
                                } else if obj["opcode"] == 0x3 {
                                    println!("received Hello packet")
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        println!("unknown content")
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

    println!("connection closed");

    receive_loop.join().unwrap_or_default();

    println!("Exiting")
}
