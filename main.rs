extern crate websocket;
extern crate execute;
extern crate json;

use std::io::stdin;
use std::sync::mpsc::channel;
use std::thread;

use websocket::client::ClientBuilder;
use websocket::{Message, OwnedMessage};

use std::process::Command;
use execute::Execute;

const CONNECTION: &'static str = "ws://127.0.0.1:2794";

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
			LEDCommand::Hold => write!(f, "hold")
		}
	}
}

fn set_color(arg: LEDCommand) {
	let mut cmd = Command::new("/usr/local/lb/LEDcolor/bin/setColor");
	cmd.arg(arg.to_string());
	let _ = cmd.execute_check_exit_status_code(0);
}

fn get_input() -> u8 {
	let mut cmd = Command::new("/usr/local/lb/ADC/bin/getADC");
	cmd.arg("-1");
	match cmd.execute_output() {
		Ok(output) => output.stdout[0],
		Err(_) => 0
	}
}

fn main() {
	println!("Connecting to {}", CONNECTION);

	let client = ClientBuilder::new(CONNECTION)
		.unwrap()
		.connect_insecure()
		.unwrap();

	println!("Successfully connected");

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
				OwnedMessage::Close(_) => {
					// Got a close message, so send a close message and return
					let _ = tx_1.send(OwnedMessage::Close(None));
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
				// Say what we received
				_ => println!("Receive Loop: {:?}", message)
			}
		}
	});

	loop {
		let mut input = String::new();

		stdin().read_line(&mut input).unwrap();

		let trimmed = input.trim();

		let message = match trimmed {
			"/close" => {
				// Close the connection
				let _ = tx.send(OwnedMessage::Close(None));
				break;
			}
			// Send a ping
			"/ping" => OwnedMessage::Ping(b"PING".to_vec()),
			// Otherwise, just send text
			_ => OwnedMessage::Text(trimmed.to_string()),
		};

		match tx.send(message) {
			Ok(()) => (),
			Err(e) => {
				println!("Main Loop: {:?}", e);
				break;
			}
		}
	}

	// We're exiting

	println!("Waiting for child threads to exit");

	let _ = send_loop.join();
	let _ = receive_loop.join();

	println!("Exited");
}