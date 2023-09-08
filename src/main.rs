mod audio;
pub mod json_processor;

use std::io::Write;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::task;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};

//use url::Url;
#[tokio::main]
async fn main() {
    let mut input = String::new();
    print!("Please enter your Roblox username!: ");
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut input).expect("Unhandled Exception!");
    let username = input.trim().to_string();
    println!("Your Roblox username is {}", username);
    let _ = task::spawn(websocket_connect(username)).await;
}

async fn websocket_connect(username: String) {
    let url = url::Url::parse("ws://client.fe2.io:8081").unwrap();

    let (ws_stream, _response) = connect_async(url).await.expect("Failed to connect");
    println!("Connection Established");

    let (mut write, read) = ws_stream.split();

    println!("sending");

    write.send(Message::Text(username)).await.unwrap();

    println!("sent");

    let player = Arc::new(audio::AudioPlayer::new());

    let read_future = read.for_each(|message| {
        let player = Arc::clone(&player);
        async move {
        
        println!("receiving...");
        let data = message.unwrap().into_data();
        tokio::io::stdout().write(&data).await.unwrap();
        if let Ok(string_data) = String::from_utf8(data.clone()) {
            println!("received: {}", string_data);
            // Process the data
            json_processor::process_data(&string_data, &player).await;
        } else {
            println!("received: Invalid UTF-8 data");
        }
}});
    

    read_future.await;
}