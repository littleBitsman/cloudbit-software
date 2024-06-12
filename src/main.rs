// cloudbit-software © 2024 by littleBitsman is licensed under CC BY-NC-SA 4.0.
// To view a copy of this license, visit http://creativecommons.org/licenses/by-nc-sa/4.0/

const DEFAULT_URL: &'static str = "wss://gateway.cloudcontrol.littlebitsman.dev/";

const INPUT_DELTA_THRESHOLD: u8 = 2;

use execute::Execute;
use futures::{channel::mpsc::channel, SinkExt, StreamExt};
use mac_address::get_mac_address;
use once_cell::sync::Lazy;
use serde::Serialize;
use serde_json::{from_str, json, to_string, Value as JsonValue};
use std::{
    fs::read_to_string,
    io::ErrorKind as IoErrorKind,
    panic::set_hook,
    process::{id as get_pid, Command},
    str::FromStr,
    time::Duration,
};
use sysinfo::{Pid, System};
use tokio::{spawn, time::sleep};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        handshake::client::{generate_key, Request},
        Error, Message,
    },
};
use url::Url;

const SYSINFO: Lazy<System> = Lazy::new(|| System::new_all());
const PID: Lazy<Pid> = Lazy::new(|| Pid::from_u32(get_pid()));

/// commands for LED as an enum
#[allow(dead_code)]
#[derive(Clone, Copy)]
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

impl TryFrom<String> for LEDCommand {
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "red" => Ok(Self::Red),
            "green" => Ok(Self::Green),
            "blue" => Ok(Self::Blue),
            "purple" => Ok(Self::Purple),
            "violet" => Ok(Self::Violet),
            "teal" => Ok(Self::Teal),
            "yellow" => Ok(Self::Yellow),
            "white" => Ok(Self::White),
            "off" => Ok(Self::Off),
            "clownbarf" => Ok(Self::Clownbarf),
            "blink" => Ok(Self::Blink),
            "hold" => Ok(Self::Hold),
            _ => Err(()),
        }
    }

    type Error = ();
}

impl Into<&str> for LEDCommand {
    fn into(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Green => "green",
            Self::Blue => "blue",
            Self::Purple => "purple",
            Self::Violet => "violet",
            Self::Teal => "teal",
            Self::Yellow => "yellow",
            Self::White => "white",
            Self::Off => "off",
            Self::Clownbarf => "clownbarf",
            Self::Blink => "blink",
            Self::Hold => "hold",
        }
    }
}

impl ToString for LEDCommand {
    fn to_string(&self) -> String {
        let s: &str = (*self).into();
        s.to_string()
    }
}

// UTILS

fn stringify(obj: impl Sized + Serialize) -> String {
    to_string(&obj).unwrap()
}

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

// MAIN LOOP
#[tokio::main]
async fn main() {
    set_led(LEDCommand::Teal);
    set_led(LEDCommand::Blink);

    let mac_address = get_mac_address()
        .expect("Failed to get MAC address")
        .expect("Failed to get MAC address");
    let cb_id_binding = read_to_string("/var/lb/id").unwrap_or("ERROR_READING_ID".to_string());
    let cb_id = cb_id_binding.trim();

    // Parse url at /usr/local/lb/cloud_client/server_url if it exists, use DEFAULT_URL if it doesn't
    let url = Url::from_str(
        &read_to_string("/usr/local/lb/cloud_client/server_url").unwrap_or(DEFAULT_URL.to_string()),
    )
    .unwrap_or(Url::from_str(DEFAULT_URL).unwrap());

    eprintln!(
        "Attempting to connect to {} ({})",
        url.to_string(),
        url.host_str().unwrap()
    );

    // initialize variables
    let mut current_input: u8 = 0; // current input (0 should be the starting value on any server implementations)
    let request = Request::get(url.as_str())
        .header("MAC-Address", mac_address.to_string())
        .header("CB-Id", cb_id)
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

    let (mut sender, mut rx) = channel::<Message>(16);
    let mut sender2 = sender.clone();

    eprintln!("Successfully connected");

    tx.send(Message::text(
        json!({
            "opcode": 0x3,
            "mac_address": mac_address.to_string(),
            "cb_id": cb_id.to_string()
        })
        .to_string(),
    ))
    .await
    .unwrap();

    set_led(LEDCommand::Green);
    set_led(LEDCommand::Hold);

    let send_loop = spawn(async move {
        while let Some(msg) = rx.next().await {
            let result = tx.send(msg).await;
            match result {
                Err(Error::AlreadyClosed | Error::ConnectionClosed) => {
                    panic!("connection closed, rebooting to attempt reconnection")
                }
                Err(Error::Io(err)) => {
                    if err.kind() == IoErrorKind::BrokenPipe {
                        panic!("connection closed, rebooting to attempt reconnection")
                    }
                }
                Err(e) => eprintln!("error on WebSocket: {}", e),
                _ => {}
            }
        }
    });

    let receive_loop = spawn(async move {
        // Receive loop
        while let Some(Ok(message)) = receiver.next().await {
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
                    eprintln!("{}", data);
                    if let Ok(parsed) = from_str::<JsonValue>(&data) {
                        if !parsed.is_object() {
                            return eprintln!("bad packet from server");
                        }
                        match parsed {
                            JsonValue::Object(ref obj) => {
                                if let Some(opcode) = obj["opcode"].as_u64() {
                                    match opcode {
                                        0x2 => {
                                            // OUTPUT
                                            if let Some(new) = obj["data"]["value"].as_u64() {
                                                set_output(new as u16);
                                            } else {
                                                eprintln!(
                                                    "bad output packet: {}",
                                                    to_string(&obj).unwrap()
                                                )
                                            }
                                        }
                                        // Any numbers that match 0xFX where X is any digit is a developer
                                        // opcode (LED set, button status, etc.)

                                        // Set LED
                                        0xF0 => {
                                            if let Some(command) = obj["led_command"].as_str() {
                                                if let Ok(led) =
                                                    LEDCommand::try_from(command.to_string())
                                                {
                                                    set_led(led);
                                                } else {
                                                    eprintln!(
                                                        "bad set LED packet: {}",
                                                        stringify(parsed)
                                                    )
                                                }
                                            } else {
                                                eprintln!(
                                                    "bad set LED packet: {}",
                                                    stringify(parsed)
                                                )
                                            }
                                        }

                                        // TODO #4 Get button (it is never sent normally)
                                        0xF1 => {}

                                        // Get system stats (e.g., memory usage, CPU usage)
                                        0xF3 => {
                                            let mut sysinfo = SYSINFO;
                                            sysinfo.refresh_cpu();
                                            sysinfo.refresh_memory();
                                            sysinfo.refresh_process(*PID);

                                            let process = sysinfo.process(*PID).unwrap();
                                            let cpu = process.cpu_usage();
                                            let mem_bytes = process.memory();
                                            let total_mem = sysinfo.total_memory();
                                            let mem_percent = (mem_bytes as f64) / (total_mem as f64);

                                            // Opcode 0xF4 is system stats (RETURNED from 0xF3)
                                            sender
                                                .send(Message::text(stringify(json!({
                                                    "opcode": 0xF4,
                                                    "stats": {
                                                        "cpu_usage": cpu,
                                                        "memory_usage": mem_bytes,
                                                        "total_memory": total_mem,
                                                        "memory_usage_percent": mem_percent
                                                    }
                                                }))))
                                                .await
                                                .unwrap();
                                        }
                                        _ => eprintln!("invalid opcode: {}", opcode),
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else {
                        eprintln!("bad packet from server")
                    }
                }
                _ => eprintln!("unknown content"),
            }
        }
    });

    set_hook(Box::new(move |v| {
        eprintln!("{}", v.to_string());
        send_loop.abort();
        receive_loop.abort();
    }));

    // Main IO loop
    loop {
        let right_now = get_input();
        if current_input.abs_diff(right_now) > INPUT_DELTA_THRESHOLD {
            current_input = right_now;
            sender2
                .send(Message::text(stringify(json!({
                    "opcode": 0x1,
                    "data": {
                        "value": current_input
                    }
                }))))
                .await
                .unwrap();
        }
        sleep(Duration::from_millis(10)).await;
    }
}
