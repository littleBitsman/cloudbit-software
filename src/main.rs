// This file is part of cloudbit-software.
//
// cloudbit-software - an alternative software for the littleBits cloudBit.
//
// Copyright (C) 2024 littleBitsman
//
// cloudbit-software is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// cloudbit-software is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the GNU General Public License for more details.
// You should have received a copy of the GNU General Public License
// along with this program. If not, see https://www.gnu.org/licenses/.

#![warn(
    clippy::undocumented_unsafe_blocks,
    clippy::absolute_paths,
    clippy::as_underscore,
    clippy::todo,
    clippy::use_self,
    clippy::semicolon_inside_block,
    clippy::uninlined_format_args
)]

const DEFAULT_URL: &str = "wss://gateway.cloudcontrol.littlebitsman.dev/";
const LOOP_DELAY_MS: u64 = 10;

/// The minimum amount that the input ADC value must change
/// before the value is considered "different" (this is an
/// attempt to reduce the effects of noise from the ADC).
const INPUT_DELTA_THRESHOLD: u16 = 2;

use futures::{channel::mpsc::channel, SinkExt, StreamExt};
use mac_address::get_mac_address;
use serde::Serialize;
use serde_json::{from_str, json, to_string, Value as JsonValue};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    fs::read_to_string,
    io::ErrorKind as IoErrorKind,
    panic::set_hook as set_panic_hook,
    process::{id as get_pid, Command},
    time::Duration,
};
use sysinfo::{ProcessesToUpdate, System};
use tokio::{spawn, time::sleep};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        handshake::client::{generate_key, Request},
        Error as WebSocketError, Message,
    },
};
use url::Url;

/// commands for LED as an enum
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    type Error = ();

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
}

impl Display for LEDCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(match self {
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
        })
    }
}

// UTILS

/// Quick way to turn a `impl Serialize` into a JSON string.
#[inline]
fn stringify(obj: impl Sized + Serialize) -> String {
    to_string(&obj).unwrap()
}

// Hardware wrappers
mod hardware;

use hardware::*;

/// set output (as 0x0000 - 0xFFFF)
/// returns success as a boolean
///
/// This does NOT have a memory-based wrapper due to the complexity of the DAC.
/// You should NEVER be calling this in very rapid succession (meaning,
/// many, many times per second for long periods of time, doing this for
/// <2 seconds at a time with long-ish breaks should be safe)
fn set_output(value: u16) -> bool {
    Command::new("/usr/local/lb/DAC/bin/setDAC")
        .arg(format!("{value:04x}"))
        .status()
        .expect("failed to execute /usr/local/lb/DAC/bin/setDAC")
        .success()
}

// MAIN LOOP
#[tokio::main]
async fn main() {
    let mac_address = get_mac_address()
        .expect("Failed to get MAC address")
        .expect("Failed to get MAC address");

    // this is like this because the borrow checker gets angry due to a value being dropped
    // but a reference to it still exists (calling trim on a String borrows it and
    // creates a &str with a reference to its data, then immediately drops the String)
    let cb_id = read_to_string("/var/lb/id").unwrap_or(String::from("ERROR_READING_ID"));
    let cb_id = cb_id.trim();

    let default_url: Url = DEFAULT_URL.parse().unwrap();

    // Parse url in /usr/local/lb/cloud_client/server_url if it exists,
    // use DEFAULT_URL if it doesn't or is not a valid URL.
    let mut url = read_to_string("/usr/local/lb/cloud_client/server_url")
        .unwrap_or(DEFAULT_URL.to_string())
        .parse()
        .unwrap_or(default_url.clone());

    // The scheme must be any of these:
    // - http (converted to ws),
    // - https (converted to wss),
    // - ws or wss
    // If it is not any of those, the error is logged and the DEFAULT_URL is used.
    // (The Url implementation returns an error if the URL is  cannot-be-a-base
    //  OR its scheme is not http, https, ws, or wss)
    match url.scheme() {
        "http" => url.set_scheme("ws").unwrap(),
        "https" => url.set_scheme("wss").unwrap(),
        "ws" | "wss" => {}
        a => {
            eprintln!("Invalid scheme {a} on cloudbit server URL, falling back to default server");
            url = default_url
        }
    }

    eprintln!(
        "Attempting to connect to {} ({})",
        url,
        url.host_str().unwrap()
    );

    // initialize variables
    let mut current_input: u16 = 0; // current input (0 should be the starting value on any server implementations)
    let request = Request::get(url.as_str())
        .header("MAC-Address", mac_address.to_string())
        .header("CB-Id", cb_id)
        .header("User-Agent", "littleARCH cloudBit")
        .header("Host", url.host_str().unwrap())
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-Websocket-Version", "13")
        .header("Sec-Websocket-Key", generate_key())
        .body(())
        .unwrap();

    let client = loop {
        led::set(LEDCommand::Teal);
        led::set(LEDCommand::Blink);
        // I wanted to avoid using Clone here but oh well
        if let Ok((client, _)) = connect_async(request.clone()).await {
            break client
        } else {
            led::set(LEDCommand::Red);
            led::set(LEDCommand::Blink);
            sleep(Duration::from_secs(2)).await;
        }
    };

    // tx: sender used internally, this is done so because tx is not Clone
    // receiver: receiver from the socket, only 1 copy is needed since its managed by 1 thread only
    let (mut tx, mut receiver) = client.split();

    // sender: sends to rx to be processed to be sent through the WebSocket
    // rx: receives all messages that need to be sent through the WebSocket via tx
    let (mut sender, mut rx) = channel::<Message>(16);
    let mut sender2 = sender.clone(); // 2 threads are running, each needs their own copy

    eprintln!("Successfully connected");

    tx.send(Message::Text(stringify(json!({
        "opcode": 0x3,
        "mac_address": mac_address.to_string(),
        "cb_id": cb_id
    }))))
    .await
    .unwrap();

    led::set(LEDCommand::Green);
    led::set(LEDCommand::Hold);

    // Captures: rx, tx
    // This handles sending messages sent over sender or sender2 to rx
    // through the WebSocket on tx.
    let send_loop = spawn(async move {
        while let Some(msg) = rx.next().await {
            let result = tx.send(msg).await;
            match result {
                Ok(()) => {}
                Err(err) => match err {
                    WebSocketError::AlreadyClosed | WebSocketError::ConnectionClosed => {
                        panic!("connection closed, rebooting to attempt reconnection")
                    }
                    WebSocketError::Io(err) => match err.kind() {
                        IoErrorKind::BrokenPipe | IoErrorKind::ConnectionReset => {
                            panic!("connection closed, rebooting to attempt reconnection")
                        }
                        IoErrorKind::OutOfMemory => panic!("!! OUT OF MEMORY !!"),
                        IoErrorKind::Interrupted => {
                            panic!("unknown interrupt, rebooting to attempt reconnection")
                        }
                        _ => {}
                    },
                    e => eprintln!("error on WebSocket: {e}"),
                },
            }
        }
    });

    // Captures: receiver, sender
    // This handles receiving and handling messages on receiver.
    let receive_loop = spawn(async move {
        // Receive loop
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(message) => match message {
                    Message::Close(frame_opt) => {
                        // we have to exit now to attempt reconnection
                        if let Some(frame) = frame_opt {
                            panic!("WebSocket closed: {frame}")
                        } else {
                            panic!("WebSocket closed, no close frame was available")
                        }
                    }
                    Message::Ping(data) => sender.send(Message::Pong(data)).await.unwrap(),
                    Message::Text(data) => {
                        // eprintln!("{data}");
                        if let Ok(JsonValue::Object(obj)) = from_str::<JsonValue>(&data) {
                            match obj["opcode"].as_u64() {
                                Some(0x2) => {
                                    // OUTPUT
                                    if let Some(new) = obj["data"]["value"].as_u64() {
                                        set_output(new as u16);
                                    } else {
                                        eprintln!("bad output packet: {}", to_string(&obj).unwrap())
                                    }
                                }

                                // Any numbers that match 0xFX where X is any digit is a developer
                                // opcode (LED set, button status, etc.)

                                // Set LED
                                Some(0xF0) => {
                                    if let Some(command) = obj["led_command"].as_str() {
                                        let command = command.replace(", ", " ").replace(",", " ");

                                        let mut chain = Vec::new();

                                        for item in command.split(" ") {
                                            if let Ok(cmd) = LEDCommand::try_from(item.to_string())
                                            {
                                                chain.push(cmd)
                                            }
                                        }
                                        led::set_many(chain);
                                    } else {
                                        eprintln!("bad set LED packet: {}", stringify(obj))
                                    }
                                }

                                // Get button (it is never sent normally)
                                Some(0xF1) => sender
                                    .send(Message::Text(stringify(json!({
                                        "opcode": 0xF2, // 0xF2 is button state (returned from 0xF1)
                                        "data": {
                                            "button": button::read()
                                        }
                                    }))))
                                    .await
                                    .unwrap(),

                                // Get system stats (e.g., memory usage, CPU usage)
                                // Note: you should NOT be polling this
                                // More notes can be found in protocol details
                                Some(0xF3) => {
                                    let mut sender = sender.clone();
                                    spawn(async move {
                                        let mut sysinfo = System::new_all();
                                        let pid = (get_pid() as usize).into();
                                        sysinfo.refresh_cpu_usage();
                                        sysinfo.refresh_memory();
                                        sysinfo.refresh_processes(ProcessesToUpdate::Some(&[pid]));

                                        sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;

                                        sysinfo.refresh_cpu_usage();

                                        let process = sysinfo.process(pid).unwrap();
                                        let cpu = process.cpu_usage();
                                        let mem_bytes = process.memory();
                                        let total_mem = sysinfo.total_memory();
                                        let mem_percent =
                                            ((mem_bytes as f64) / (total_mem as f64)) * 100.0;
                                        let cpu_temp_kelvin = adc::read_temp();

                                        // Opcode 0xF4 is system stats (RETURNED from 0xF3)
                                        sender
                                            .send(Message::Text(stringify(json!({
                                                "opcode": 0xF4,
                                                "stats": {
                                                    "cpu_usage": cpu,
                                                    "memory_usage": mem_bytes,
                                                    "total_memory": total_mem,
                                                    "memory_usage_percent": mem_percent,
                                                    "cpu_temp_kelvin": cpu_temp_kelvin
                                                }
                                            }))))
                                            .await
                                            .unwrap();
                                    });
                                }
                                Some(opcode) => eprintln!("invalid opcode: {opcode}"),
                                None => {}
                            }
                        } else {
                            eprintln!("bad packet from server: {data}")
                        }
                    }
                    _ => eprintln!("unknown content"),
                },
                Err(err) => match err {
                    WebSocketError::Io(err) => match err.kind() {
                        IoErrorKind::BrokenPipe | IoErrorKind::ConnectionReset => {
                            panic!("connection closed, rebooting to attempt reconnection")
                        }
                        IoErrorKind::OutOfMemory => panic!("!! OUT OF MEMORY !!"),
                        IoErrorKind::Interrupted => {
                            panic!("unknown interrupt, rebooting to attempt reconnection")
                        }
                        _ => {}
                    },
                    e => eprintln!("error on WebSocket: {e}"),
                },
            }
        }
    });

    set_panic_hook(Box::new(move |v| {
        eprintln!("{v}");
        send_loop.abort();
        receive_loop.abort();
        // Turns out the memory mapping is removed after the process exits lol
        // hardware::cleanup_all();
    }));

    hardware::init_all()
        .map_err(|(origin, err)| format!("failed to initialize {origin}: {err}"))
        .unwrap();

    // Main IO loop
    loop {
        let right_now = adc::read();
        if current_input.abs_diff(right_now) > INPUT_DELTA_THRESHOLD {
            current_input = right_now;
            sender2
                .send(Message::Text(stringify(json!({
                    "opcode": 0x1,
                    "data": {
                        "value": current_input
                    }
                }))))
                .await
                .unwrap();
        }
        sleep(Duration::from_millis(LOOP_DELAY_MS)).await;
    }
}
