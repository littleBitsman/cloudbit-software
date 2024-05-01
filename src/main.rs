use std::{
    fs::read_to_string,
    panic::catch_unwind,
    process::Command,
    thread::{self, sleep},
    time,
    net::UdpSocket
};
use execute::Execute;

const LOCAL_PORT: &'static u16 = &3001;
// const REMOTE_ADDR: &'static str = "192.168.1.155:3000";
// const REMOTE_ADDR: &'static str = "127.0.0.1:3000";

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

fn start(url: &str, mac_address: &str) {
    let mac_bytes = mac_address.as_bytes().to_owned();
    let mac_bytes_2 = mac_bytes.clone();

    let socket = UdpSocket::bind(format!("127.0.0.1:{}", LOCAL_PORT)).expect("Failed to bind");
    socket.connect(url).expect("Failed to connect");
    socket
        .send(format!("{:?}identify", mac_bytes).as_bytes())
        .expect("[identify] failed to send identify packet");

    let clone = socket.try_clone().unwrap();

    thread::spawn(move || loop {
        let _ = catch_unwind(|| {
            let mut buf = [0; 15];
            let amount = clone.recv(&mut buf).unwrap();
            let (mac_buf, mut main_buf) = buf.split_at_mut(12);
            main_buf = &mut main_buf[..(amount - 12)];
            if mac_bytes_2 != mac_buf {
                return println!(
                    "received msg intended for another cloudbit. expected: {:?}, got: {:?}",
                    String::from_utf8_lossy(&mac_bytes_2),
                    String::from_utf8_lossy(&mac_buf)
                );
            }
            let main = String::from_utf8_lossy(main_buf);

            let (cmd, arg) = main.split_at(1);

            match cmd {
                "O" => {
                    let num = u16::from_str_radix(arg, 10).unwrap();
                    set_output(num);
                    println!("received output packet: {}", num);
                }
                _ => {}
            }
        });
    });

    let mut current = 0;

    loop {
        if get_input() != current {
            current = get_input();
            socket
                .send(format!("{:?}I:{}", mac_bytes, current).as_bytes())
                .expect("[input] failed to send updated input");
        }
    }
}

fn main() {
    let binding = read_to_string("/var/lb/mac").unwrap_or("000000000000".to_owned());
    let mac_address = binding.split_at(12).0;
    
    let url = &read_to_string("/usr/local/lb/cloud_client/server_url").unwrap_or("127.0.0.1:3000".to_owned());

    set_led(LEDCommand::Green);
    set_led(LEDCommand::Blink);
    loop {
        let result = std::panic::catch_unwind(|| start(url, mac_address));
        match result {
            Ok(()) => {}
            Err(_) => {
                println!("error occured; attempting to restart");
                set_led(LEDCommand::Red);
                set_led(LEDCommand::Blink);
                sleep(time::Duration::from_secs(2));
                set_led(LEDCommand::Green);
                break;
            }
        }
    }
}
