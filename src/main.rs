use std::io::{self, Write};
use tokio::io::{AsyncWriteExt, Result};
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
    task::spawn(websocket_connect(username)).await;
}

async fn websocket_connect(username: String) {
    let url = url::Url::parse("ws://client.fe2.io:8081").unwrap();

    let (ws_stream, _response) = connect_async(url).await.expect("Failed to connect");
    println!("Connection Established");

    let (mut write, read) = ws_stream.split();

    println!("sending");

    write.send(Message::Text(username)).await.unwrap();

    println!("sent");

    let read_future = read.for_each(|message| async {
        println!("receiving...");
        let data = message.unwrap().into_data();
        tokio::io::stdout().write(&data).await.unwrap();
        if let Ok(string_data) = String::from_utf8(data.clone()) {
            println!("received: {}", string_data);
        } else {
            println!("received: Invalid UTF-8 data");
        }
    });

    read_future.await;
}