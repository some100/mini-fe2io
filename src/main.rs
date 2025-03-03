mod audio;
pub mod json_processor;

use anyhow::{Context, Error};
use clap::{Arg, Command};
use futures_util::{SinkExt, StreamExt};
use rodio::{OutputStream, Sink};
use std::io::Write;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::channel;
use tokio::task;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Command Line Arguments
    let matches = Command::new("Mini FE2IO")
        .version("0.1.2")
        .author("Abraham Richard Sunjaya")
        .about("A miniaturized version of FE2.IO written in Rust, and independent of any web browsers.")
        .arg(
            Arg::new("username")
                .short('u')
                .long("username")
                .help("Your Roblox Username"))
        .arg(
            Arg::new("volume")
                .short('v')
                .long("volume")
                .help("Volume"))
        .get_matches();

    // Check for username
    let username = if let Some(user) = matches.get_one::<String>("username") {
        user.to_string()
    }
    else {
        let mut input = String::new();
        print!("Please enter your Roblox username: ");
        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    };

    // Check for volume
    let volume= if let Some(volume) = matches.get_one::<String>("volume") {
        match volume.parse::<f32>() {
            Ok(v) if (0.0..=100.0).contains(&v) => v / 100.0,
            _ => {
                eprintln!("Volume must be a number between 0 to 100! Defaulting to 70%");
                0.7
            }
        }
    } else {
        0.7
    };

    websocket_connect(username, volume).await?;
    Ok(())
}

async fn websocket_connect(username: String, volume: f32) -> Result<(), Error> {
    // this does way more than websocket connect lol
    let (ws_stream, _response) = connect_async("ws://client.fe2.io:8081").await?;
    println!("Connection established");

    let (mut write, mut read) = ws_stream.split();
    write.send(Message::Text(username)).await?;

    println!("Sent username to server");

    let (tx, rx) = channel(2);
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    sink.set_volume(volume);
    task::spawn(audio::audio_loop(rx, sink));

    loop {
        let message = read
            .next()
            .await
            .context("Couldn't receive messages from server")?;
        println!("receiving...");
        let data = message?.into_data();
        tokio::io::stdout().write(&data).await?;
        if let Ok(string_data) = String::from_utf8(data.clone()) {
            println!("received: {}", string_data);
            // Process the data
            json_processor::process_data(&string_data, tx.clone()).await?;
        } else {
            println!("received: Invalid UTF-8 data");
        }
    }
}
