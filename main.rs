extern crate execute;
extern crate json;
extern crate websocket;

use core::time;
use std::sync::mpsc::channel;
use std::thread::{self, sleep};

use json::object;
use websocket::client::ClientBuilder;
use websocket::header::Headers;
use websocket::{Message, OwnedMessage};

use execute::Execute;
use std::process::Command;

const CONNECTION: &'static str = "ws://192.168.1.155:3000/";
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

fn set_color(arg: LEDCommand) {
    return;

    let mut cmd = Command::new("/usr/local/lb/LEDcolor/bin/setColor");
    cmd.arg(arg.to_string());
    cmd.execute_check_exit_status_code(0).unwrap()
}

fn get_input() -> u8 {
    return 5;

    let mut cmd = Command::new("/usr/local/lb/ADC/bin/getADC");
    cmd.arg("-1");
    match cmd.execute_output() {
        Ok(output) => output.stdout[0],
        Err(_) => 0,
    }
}

fn set_output(value: u16) {
    println!("output cmd: 0x{:04x}", value);
    return;

    let mut cmd = Command::new("/usr/local/lb/DAC/bin/setDAC");
    cmd.arg(format!("0x{:04x}", value));
}

fn main() {
    set_color(LEDCommand::Green);
    set_color(LEDCommand::Blink);
    loop {
        let t = thread::spawn(start);
        match t.join() {
            Ok(()) => {}
            Err(_) => {
                set_color(LEDCommand::Red);
                set_color(LEDCommand::Blink);
                sleep(time::Duration::from_secs(2));
                set_color(LEDCommand::Green);
            }
        }
    }
}

fn start() {
    println!("Connecting to {}", CONNECTION);

    let mut headers = Headers::new();
    headers.append_raw("MAC-Address", "test".into());
    headers.append_raw("User-Agent", "littleARCH cloudBit".into());

    let client = ClientBuilder::new(CONNECTION)
        .unwrap()
        .custom_headers(&headers)
        .connect_insecure()
        .unwrap();

    println!("Successfully connected");

    set_color(LEDCommand::Hold);

    let mut current_input: u8 = 0;

    let (mut receiver, mut sender) = client.split().unwrap();

    let (tx, rx) = channel();

    let tx_1 = tx.clone();

    let send_loop = thread::spawn(move || {
        loop {
            // Send loop
            let message = match rx.recv() {
                Ok(m) => m,
                Err(e) => {
                    println!("Send Loop: {:?}", e);
                    return;
                }
            };
            match message {
                OwnedMessage::Close(_) => {
                    let _ = sender.send_message(&message);
                    // If it's a close message, just send it and then return.
                    return;
                }
                _ => (),
            }
            // Send the message
            match sender.send_message(&message) {
                Ok(()) => (),
                Err(e) => {
                    println!("Send Loop: {:?}", e);
                    let _ = sender.send_message(&Message::close());
                    return;
                }
            }
        }
    });

    let receive_loop = thread::spawn(move || {
        // Receive loop
        for message in receiver.incoming_messages() {
            let message = match message {
                Ok(m) => m,
                Err(e) => {
                    println!("Receive Loop: {:?}", e);
                    let _ = tx_1.send(OwnedMessage::Close(None));
                    return;
                }
            };
            match message {
                OwnedMessage::Close(a) => {
                    // Got a close message, so send a close message and return
                    let _ = tx_1.send(OwnedMessage::Close(a));
                    return;
                }
                OwnedMessage::Ping(data) => {
                    match tx_1.send(OwnedMessage::Pong(data)) {
                        // Send a pong in response
                        Ok(()) => (),
                        Err(e) => {
                            println!("Receive Loop: {:?}", e);
                            return;
                        }
                    }
                }
                OwnedMessage::Text(data) => {
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
							if obj["opcode"] == 0x2 { // OUTPUT
								let new = obj["data"]["value"].as_u16().expect("bad output packet from server");
								set_output(new);
							} else if obj["opcode"] == 0x3 {
								println!("received Hello packet")
							}
						},
						_ => {}
					}
                }
                _ => {
                    println!("unknown content")
                }
            }
        }
    });

    loop {
        let right_now = get_input();
        if right_now != current_input {
            current_input = right_now;
            let success = tx
                .send(OwnedMessage::Text(json::stringify(object!{
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

    let _ = send_loop.join();
    let _ = receive_loop.join();

    println!("Exiting")
}
