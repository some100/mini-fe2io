mod audio;
mod json_processor;
mod keybind;

use anyhow::{Context, Error};
use clap::Parser;
use futures_util::{SinkExt, StreamExt, stream::SplitStream};
use rodio::{OutputStream, Sink};
use std::io::{self, Write};
use tokio::{
    net::TcpStream,
    sync::mpsc::{Sender, channel},
    task,
};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::protocol::Message,
};

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
    /// Server
    #[arg(
        short = 's',
        long = "server",
        default_value = "ws://client.fe2.io:8081"
    )]
    server: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Command Line Arguments
    let args = Args::parse();

    // Check for username
    #[allow(clippy::single_match_else, reason = "this looks more natural than the equivalent if let")]
    let username = match args.username {
        Some(username) => username,
        None => {
            let mut input = String::new();
            print!("Please enter your roblox username: ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut input)?;
            input.trim().to_owned()
        }
    };

    // Check for volume
    let volume = args.volume.clamp(0.0, 100.0) / 100.0; // clamp volume between 0% and 100%

    // Create a connection to the server
    let read = websocket_connect(args.server, username).await?;

    // Create an mpsc channel for communication between the JSON processor and the audio loop
    let (tx, rx) = channel(32);
    let (volume_tx, volume_rx) = channel(4);

    // Create Sink for audio device and set volume to the volume passed in argument
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    sink.set_volume(volume);

    // Spawn separate task for handling audio events
    task::spawn(audio::audio_loop(rx, volume_rx, sink));
    // Spawn separate task for handling websocket events
    task::spawn(websocket_loop(tx.clone(), read));

    keybind::keybind_listen(tx, volume_tx, volume).await?;
    Ok(())
}

async fn websocket_connect(
    server: String,
    username: String,
) -> Result<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>, Error> {
    // this does a reasonable amount of work for websocket connect
    let (ws_stream, _response) = connect_async(server).await?;
    println!("Connection established");

    let (mut write, read) = ws_stream.split();
    write.send(Message::Text(username)).await?;

    println!("Sent username to server");

    Ok(read)
}

async fn websocket_loop(
    tx: Sender<String>,
    mut read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> Result<(), Error> {
    loop {
        let message = read
            .next()
            .await
            .context("Couldn't receive messages from server")?;
        let data = message?.into_text()?;
        println!("Got message {data}");
        json_processor::process_data(&data, &tx).await?;
    }
}
