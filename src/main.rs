use execute::Execute;
use mac_address::get_mac_address;
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    fs::read_to_string,
    net::UdpSocket,
    panic::catch_unwind,
    process::Command,
    sync::Arc,
    thread::{sleep, spawn},
    time,
};

const LOCAL_PORT: &'static u16 = &3000;
// const REMOTE_ADDR: &'static str = "192.168.1.155:3000";

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

impl Display for LEDCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}",
            match self {
                LEDCommand::Red => "red",
                LEDCommand::Green => "green",
                LEDCommand::Blue => "blue",
                LEDCommand::Purple => "purple",
                LEDCommand::Violet => "purple",
                LEDCommand::Teal => "teal",
                LEDCommand::Yellow => "yellow",
                LEDCommand::White => "white",
                LEDCommand::Off => "off",
                LEDCommand::Clownbarf => "clownbarf",
                LEDCommand::Blink => "blink",
                LEDCommand::Hold => "hold",
            }
        )
    }
}

mod hardware;

use hardware::*;

fn set_output(value: u16) -> bool {
    let mut cmd = Command::new("/usr/local/lb/DAC/bin/setDAC");
    cmd.arg(format!("{:04x}", value));
    cmd.execute_check_exit_status_code(0).is_ok()
}

fn start(url: &str) {
    let mac_bytes = get_mac_address()
        .expect("Failed to get MAC address")
        .expect("Failed to get MAC address")
        .bytes()
        .to_vec();
    let mac_bytes_2 = mac_bytes.clone();

    let socket = Arc::new(
        UdpSocket::bind(format!("127.0.0.1:{}", LOCAL_PORT)).expect("[socket] failed to bind"),
    );
    println!("bound to UDP port");
    socket
        .connect(url.trim())
        .expect("[socket] failed to connect");
    println!("connected");
    socket
        .send(&mac_bytes)
        .expect("[identify] failed to send identify packet");
    let clone = socket.clone();

    spawn(move || loop {
        catch_unwind(|| {
            let mut buf = [0; 15];
            clone.recv(&mut buf).unwrap();

            let (mac_buf, main_buf) = buf.split_at(6);

            if !mac_bytes_2
                .iter()
                .enumerate()
                .all(|(i, v)| Some(v) == mac_buf.get(i))
            {
                return println!(
                    "[socket] received msg intended for another cloudbit. expected: {:?}, got: {:?}",
                    mac_bytes_2, mac_buf
                );
            }

            let (cmd, buf) = (main_buf[0], main_buf.split_at(1).1);

            match cmd {
                79 => {
                    // 79 = "O"
                    let num = u16::from_le_bytes([buf[0], buf[1]]);
                    set_output(num);
                    println!("[output] received packet: {}", num);
                }
                66 => {
                    // 66 = "B"
                    let mut msg = mac_bytes_2.clone();
                    msg.push(66);
                    msg.push(button::read() as u8);

                    clone.send(&msg).expect("[button] failed to send button state packet");
                }
                _ => println!("{:?}", buf),
            }
        })
        .ok();
    });

    let mut current = 0;
    let mut msg = mac_bytes.clone();
    msg.push(73); // "I" = 73
    msg.push(0);

    loop {
        let now = adc::read();
        if now != current {
            current = now;
            msg[13] = current;
            socket
                .send(&msg)
                .expect("[input] failed to send updated input");
        }
    }
}

fn main() {
    let url = read_to_string("/usr/local/lb/cloud_client/udp_server_url")
        .expect("server URL is required since there is no default");

    hardware::init_all().map_err(|(origin, err)| format!("failed to initialize {origin}: {err}")).unwrap();

    led::set(LEDCommand::Green);
    led::set(LEDCommand::Blink);
    loop {
        let result = catch_unwind(|| start(&url));
        match result {
            Ok(()) => {}
            Err(_) => {
                eprintln!("error occured; attempting to restart");
                led::set(LEDCommand::Red);
                led::set(LEDCommand::Blink);
                sleep(time::Duration::from_secs(2));
            }
        }
    }
}
