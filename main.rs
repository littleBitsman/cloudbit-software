extern crate execute;
extern crate json;

use json::{object, JsonValue};

use std::{
    fs,
    process::Command,
    str::from_utf8,
    thread::{self, sleep},
    time,
};

use crate::execute::Execute;

use std::net::UdpSocket;

const LOCAL_ADDR: &'static str = "127.0.0.1:3000";
const REMOTE_ADDR: &'static str = "192.168.1.155:3000";

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

fn set_led(arg: LEDCommand) -> bool {
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

fn start(mac_address: &str, cb_id: &str) {
    let socket = UdpSocket::bind(LOCAL_ADDR).expect("Failed to bind");
    socket.connect(REMOTE_ADDR).expect("Failed to connect");
    socket
        .send(
            json::stringify(object! {
                opcode: 0x3,
                mac: mac_address,
                id: cb_id
            })
            .as_bytes(),
        )
        .expect("Failed to identify");

    let clone = socket.try_clone().unwrap();

    thread::spawn(move || loop {
        let mut buf = [0; 1000];
        let amount = clone.recv(&mut buf).unwrap();
        let data = json::parse(from_utf8(&mut buf[..amount]).unwrap()).unwrap();

        match data {
            JsonValue::Object(obj) => {
                let opcode = obj["opcode"].as_u8().unwrap_or(0);
                if opcode == 0x2 {
                    let new = obj["data"]["value"]
                        .as_u16()
                        .expect("bad output packet from server");
                    set_output(new);
                    println!("new output {}", new);
                } else if opcode == 0x4 {
                    println!("received hello packet from server")
                }
            }
            _ => {
                println!("received invalid data, ignoring")
            }
        }
    });

    let mut current = 0;

    loop {
        if get_input() != current {
            current = get_input();
            socket
                .send(
                    json::stringify(object! {
                        opcode: 0x1
                    })
                    .as_bytes(),
                )
                .expect("[input] failed to send updated input");
        }
    }
}

fn main() {
    let mac_address = fs::read_to_string("/var/lb/mac").unwrap_or("ERROR_READING_MAC".to_string());
    let cb_id = fs::read_to_string("/var/lb/id").unwrap_or("ERROR_READING_ID".to_string());

    set_led(LEDCommand::Green);
    set_led(LEDCommand::Blink);
    loop {
        let result = std::panic::catch_unwind(|| start(&mac_address, &cb_id));
        match result {
            Ok(()) => {}
            Err(_) => {
                set_led(LEDCommand::Red);
                set_led(LEDCommand::Blink);
                sleep(time::Duration::from_secs(2));
                set_led(LEDCommand::Green);
                break;
            }
        }
    }
}
