mod audio;
pub mod json_processor;

use std::io::Write;
use tokio::io::AsyncWriteExt;
use tokio::task;
use tokio::sync::mpsc::channel;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use rodio::{OutputStream, Sink};
use anyhow::{Context, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut input = String::new();
    print!("Please enter your Roblox username: ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;
    let username = input.trim().to_string();
    websocket_connect(username).await?;
    Ok(())
}

async fn websocket_connect(username: String) -> Result<(), Error> { // this does way more than websocket connect lol
    let (ws_stream, _response) = connect_async("ws://client.fe2.io:8081").await?;
    println!("Connection established");

    let (mut write, mut read) = ws_stream.split();
    write.send(Message::Text(username)).await?;

    println!("Sent username to server");

    let (tx, rx) = channel(2);
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    task::spawn(audio::audio_loop(rx, sink));

    loop {
        let message = read.next().await
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