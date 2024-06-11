// cloudbit-software © 2024 by littleBitsman is licensed under CC BY-NC-SA 4.0.
// To view a copy of this license, visit http://creativecommons.org/licenses/by-nc-sa/4.0/

const DEFAULT_URL: &'static str = "wss://gateway.cloudcontrol.littlebitsman.dev/";

use execute::Execute;
use futures::{channel::mpsc::channel, SinkExt, StreamExt};
use json::{object, parse, stringify, JsonValue};
use mac_address::get_mac_address;
use once_cell::sync::Lazy;
use std::{
    fmt::{Display, Formatter, Result as FmtResult}, fs::read_to_string, os::unix::net::UnixDatagram, panic::set_hook, process::Command, str::FromStr
};
use tokio::spawn;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        handshake::client::{generate_key, Request},
        Message,
    }
};
use url::Url;

const ADC_SOCKET: Lazy<UnixDatagram> = Lazy::new(|| {
    let socket = UnixDatagram::unbound().unwrap();
    socket.connect("/var/lb/ADC_socket").unwrap();
    socket
});
const DAC_SOCKET: Lazy<UnixDatagram> = Lazy::new(|| {
    let socket = UnixDatagram::unbound().unwrap();
    socket.connect("/var/lb/DAC_socket").unwrap();
    socket
});
// /var/lb/BUTTON_socket - button; not sure if needed
// /var/lb/SET_COLOR_socket - LED; not sure if needed

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
    Hold
}

/// allows formatting an `LEDCommand` into a string
impl Display for LEDCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}",
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
                Self::Hold => "hold"
            }
        )
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
    match ADC_SOCKET.send(b"-1") {
        Err(_) => return 0,
        _ => {}
    };
    let mut buf = vec![];
    match ADC_SOCKET.recv(&mut buf) {
        Err(_) => return 0,
        _ => {}
    }
    u8::from_str_radix(String::from_utf8_lossy(&buf).split_whitespace().nth(0).unwrap_or("0"), 10).unwrap_or(0)
    /*
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
    */
}

/// set output (as 0x0000 - 0xFFFF)
/// returns success as a boolean
fn set_output(value: u16) -> bool {
    DAC_SOCKET.send(format!("{:04x}", value).as_bytes()).is_ok()
    /*
    let mut cmd = Command::new("/usr/local/lb/DAC/bin/setDAC");
    cmd.arg(format!("{:04x}", value));
    cmd.execute_check_exit_status_code(0).is_ok()
    */
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

    let (mut sender, mut rx) = channel::<Message>(8);
    let mut sender2 = sender.clone();

    eprintln!("Successfully connected");

    tx.send(Message::text(stringify(object! {
        opcode: 0x3,
        mac_address: mac_address.to_string(),
        cb_id: cb_id.to_string()
    })))
    .await
    .unwrap();

    set_led(LEDCommand::Green);
    set_led(LEDCommand::Hold);

    let send_loop = spawn(async move {
        while let Some(msg) = rx.next().await {
            let result = tx.send(msg).await;
            match result {
                Err(e) => eprintln!("error {}", e),
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
                    if let Ok(parsed) = parse(&data) {
                        if !parsed.is_object() {
                            return eprintln!("bad packet from server");
                        }
                        match parsed {
                            JsonValue::Object(obj) => {
                                let opcode = obj["opcode"].as_u16().unwrap_or(0);

                                match opcode {
                                    0x2 => {
                                        // OUTPUT
                                        let new = obj["data"]["value"]
                                            .as_u16()
                                            .expect("bad output packet from server");
                                        set_output(new);
                                    }
                                    _ => eprintln!("invalid opcode ({})", opcode),
                                }
                            }
                            _ => {}
                        }
                    } else {
                        eprintln!("bad packet from server")
                    }
                }
                _ => eprintln!("unknown content")
            }
        }
    });

    set_hook(Box::new(move |v| {
        send_loop.abort();
        receive_loop.abort();
        println!("{}", v.to_string())
    }));

    // Main IO loop
    loop {
        let right_now = get_input();
        if right_now != current_input {
            current_input = right_now;
            sender2
                .send(Message::text(stringify(object! {
                    opcode: 0x1,
                    data: object! {
                        value: current_input
                    }
                })))
                .await
                .unwrap();
        }
    }
}
