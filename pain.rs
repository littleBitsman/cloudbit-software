use async_tungstenite::tungstenite::protocol::Message;
use async_tungstenite::tungstenite::client::AutoStream;
use async_tungstenite::connect_async;
use futures::stream::StreamExt;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};
use url::Url;

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

// Your LEDCommand implementation...

// MAIN LOOP
#[tokio::main]
async fn main() {
    set_led(LEDCommand::Teal);
    set_led(LEDCommand::Blink);

    let url = std::fs::read_to_string("/usr/local/lb/cloud_client/server_url")
        .unwrap_or_else(|_| "ws://chiseled-private-cauliflower.glitch.me/".to_owned());

    loop {
        match start(url.as_str()).await {
            Ok(_) => println!("you closed the connection somehow why??"),
            Err(err) => {
                println!("error {:?}", err);
                set_led(LEDCommand::Red);
                set_led(LEDCommand::Blink);
                sleep(Duration::from_secs(2)).await;
                set_led(LEDCommand::Teal);
            }
        }
    }
}

// MAIN SOCKET
async fn start(conf: &str) -> Result<(), tungstenite::Error> {
    let url = Url::parse(conf)?;

    println!(
        "Attempting to connect to {} ({})",
        url.to_string(),
        url.host_str().unwrap()
    );

    set_led(LEDCommand::Hold);

    let tcp_stream = TcpStream::connect(url.host_str().unwrap()).await?;
    let (ws_stream, _) = connect_async(url.as_str(), tcp_stream).await?;

    println!("Successfully connected");

    set_led(LEDCommand::Green);

    let (mut write_half, mut read_half) = ws_stream.split();

    let receive_loop = tokio::spawn(async move {
        // Receive loop
        while let Some(message) = read_half.next().await {
            match message {
                Ok(message) => match message {
                    Message::Close(a) => {
                        // Got a close message, so send a close message and return
                        let _ = write_half.send(Message::Close(a)).await;
                    }
                    Message::Ping(data) => {
                        match write_half.send(Message::Pong(data)).await {
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
                        // Process the received message
                    }
                    _ => {
                        eprintln!("unknown content");
                    }
                },
                Err(e) => {
                    eprintln!("Receive Error: {:?}", e);
                    return;
                }
            }
        }
    });

    let mut current_input: u8 = 0;
    loop {
        let right_now = get_input();
        println!("input {}", right_now);
        if right_now != current_input {
            current_input = right_now;
            if let Err(e) = write_half
                .send(Message::Text(json::stringify(object! {
                    opcode: 0x1,
                    data: object! {
                        value: current_input
                    }
                })))
                .await
            {
                eprintln!("Send Error: {:?}", e);
                break;
            }
        }
    }

    if let Err(e) = write_half.send(Message::Close(None)).await {
        eprintln!("Close Error: {:?}", e);
    }

    println!("connection closed");

    if let Err(e) = receive_loop.await {
        eprintln!("Receive Loop Error: {:?}", e);
    }

    Ok(())
}