// cloudbit-software © 2024 by littleBitsman is licensed under CC BY-NC-SA 4.0. 
// To view a copy of this license, visit http://creativecommons.org/licenses/by-nc-sa/4.0/

use execute::Execute;
use futures::channel::mpsc::channel;
use futures::{SinkExt, StreamExt};
use json::object;
use std::panic::set_hook;
use std::process::Command;
use std::str::FromStr;
use tokio::spawn;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        handshake::client::{generate_key, Request},
        Message,
    },
};
use url::Url;

/// commands for LED as an enum
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

/// allows formatting an `LEDCommand` into a string
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

/// the raw form of `set_led()`, directly passes `str` to `/usr/local/lb/LEDcolor/bin/setColor`
/// returns success as a boolean
fn set_led_raw(str: String) -> bool {
    let mut cmd = Command::new("/usr/local/lb/LEDcolor/bin/setColor");
    cmd.arg(str);
    cmd.execute_check_exit_status_code(0).is_ok()
}

/// set led using `LEDCommand`
/// returns success as a boolean
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
            let num = lines.split("\n").next().unwrap_or("0");
            u8::from_str(num.trim()).unwrap()
        }
        Err(_) => 0,
    }
}

/// set output (as 0x0000 - 0xFFFF)
/// returns success as a boolean
fn set_output(value: u16) -> bool {
    let mut cmd = Command::new("/usr/local/lb/DAC/bin/setDAC");
    cmd.arg(format!("{:04x}", value));
    cmd.execute_check_exit_status_code(0).is_ok()
}

const DEFAULT_URL: &'static str = "wss://gateway.cloudcontrol.littlebitsman.dev/";

// MAIN LOOP
#[tokio::main]
async fn main() {
    set_led(LEDCommand::Teal);
    set_led(LEDCommand::Blink);

    // Parse url at /usr/local/lb/cloud_client/server_url if it exists, use DEFAULT_URL if it doesn't
    let url = Url::from_str(&std::fs::read_to_string("/usr/local/lb/cloud_client/server_url").unwrap_or(DEFAULT_URL.to_string()))
        .unwrap_or(Url::from_str(DEFAULT_URL).unwrap());

    println!(
        "Attempting to connect to {} ({})",
        url.to_string(),
        url.host_str().unwrap()
    );

    // initalize variables

    let mut current_input: u8 = 0; // current input (0 should be the starting value on any server implementations)
    let request = Request::get(url.as_str())
        // .header("MAC-Address", conf.mac_address.as_str())
        // .header("CB-Id", conf.cb_id.as_str())
        .header("User-Agent", "littleARCH cloudBit")
        .header("Host", url.host_str().unwrap())
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-Websocket-Version", "13")
        .header("Sec-Websocket-Key", generate_key().as_str())
        .body(())
        .unwrap();

    let (client, _) = connect_async(request).await.unwrap();

    let (mut tx, mut receiver) = client.split();

    let (mut sender, mut rx) = channel::<Message>(8);
    let mut sender2 = sender.clone();

    println!("Successfully connected");

    set_led(LEDCommand::Green);
    set_led(LEDCommand::Hold);

    let send_loop = spawn(async move {
        while let Some(msg) = rx.next().await {
            let result = tx.send(msg).await;
            match result {
                Ok(()) => println!("success"),
                Err(e) => println!("error {}", e),
            }
        }
    });

    let receive_loop = spawn(async move {
        // Receive loop
        while let Ok(message) = receiver.next().await.unwrap() {
            match message {
                Message::Close(a) => {
                    // Got a close message, so send a close message and return
                    let _ = sender.send(Message::Close(a));
                }
                Message::Ping(data) => {
                    match sender.send(Message::Pong(data)).await {
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
                        return eprintln!("bad packet from server");
                    }
                    let parsed = r.unwrap();
                    if !parsed.is_object() {
                        return eprintln!("bad packet from server");
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
                                    if let Some(c) = obj["color"].as_str() {
                                        set_led_raw(c.to_lowercase());
                                    }
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
    });

    set_hook(Box::new(move |_| {
        send_loop.abort();
        receive_loop.abort();
    }));

    // Main IO loop
    loop {
        let right_now = get_input();
        if right_now != current_input {
            println!("input {}", right_now);
            println!("input {}", right_now);
            current_input = right_now;
            sender2
                .send(Message::Text(json::stringify(object! {
                    opcode: 0x1,
                    data: object! {
                        value: current_input
                    }
                })))
                .await
                .unwrap();
        }
    }
    /*
    // Send close 
    sender2.send(Message::Close(None)).await.unwrap();

    println!("connection closed");

    receive_loop.await.unwrap_or_default();
    send_loop.await.unwrap_or_default();
    */
}
