mod audio;
pub mod json_processor;

use anyhow::{Context, Error};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use rodio::{OutputStream, Sink};
use std::io::Write;
use tokio::sync::mpsc::channel;
use tokio::task;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

/// A miniaturized version of FE2.IO written in Rust, and independent of any web browsers.
#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    /// Your Roblox Username
    #[arg(short = 'u', long = "username")]
    username: Option<String>,
    /// Volume
    #[arg(short = 'v', long = "volume", default_value_t = 70.0)]
    volume: f32,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Command Line Arguments
    let args = Args::parse();

    // Check for username
    let username = if let Some(user) = args.username {
        user
    } else {
        let mut input = String::new();
        print!("Please enter your Roblox username: ");
        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    };

    // Check for volume
    let volume = args.volume.clamp(0.0, 100.0) / 100.0; // clamp volume between 0% and 100%

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
        let data = message?.into_text()?;
        println!("Got message {}", data);
        json_processor::process_data(&data, tx.clone()).await?;
    }
}
