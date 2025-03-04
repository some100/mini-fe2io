use std::io::Cursor;
use tokio::sync::mpsc::Receiver;
use reqwest::Client;
use rodio::{Decoder, Sink};
use anyhow::{Context, Error};

pub async fn audio_loop(mut rx: Receiver<String>, mut volume_rx: Receiver<f32>, sink: Sink) -> Result<(), Error> {
    let client = Client::new();
    let mut volume = sink.volume();
    loop {
        let input = rx.recv().await
            .context("Audio channel closed")?;
        match input.as_str() {
            "volume" => { // internal event
                let received_volume = volume_rx.recv().await // this shouldn't stall the audio loop since volume_rx should already have a message by the time volume is received
                    .context("Received volume from keybind listener was not of type f32")?; // this shouldnt ever happen
                sink.set_volume(received_volume);
                volume = received_volume;
                println!("Volume set to {}", volume * 100.0);
            },
            "died" => sink.set_volume(volume / 2.0),
            "left" => sink.stop(),
            _ => {
                sink.set_volume(volume);
                play_audio(input, &client, &sink).await?;
            },
        }
    }
}

pub async fn play_audio(url: String, client: &Client, sink: &Sink) -> Result<(), Error> {
    println!("Got request to play audio {}", url);
    // Stop the current playback, if any
    sink.stop();

    // Get response from URL
    let response = client.get(url)
        .send()
        .await?
        .bytes()
        .await?;

    println!("Got response");

    // Play the downloaded audio
    let cursor = Cursor::new(response);
    let decoder = Decoder::new(cursor)?;
    sink.append(decoder);
    println!("Playback started");
    Ok(())
}