use anyhow::{Context, Error};
use reqwest::Client;
use rodio::{Decoder, Sink, Source};
use serde_json::{json, Value};
use std::{
    io::Cursor, 
    path::PathBuf,
};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    sync::mpsc::Receiver,
    time::Instant,
};
use serde::{Serialize, Deserialize};
use random_string::generate;

const CHARSET: &str = "abcdefghijklmnopqrstuvwxyz1234567890";

#[derive(Serialize, Deserialize)]
struct AudioCache {
    urls: Vec<String>,
    files: Vec<String>,
}

pub async fn audio_loop(
    mut rx: Receiver<String>,
    mut volume_rx: Receiver<f32>,
    sink: Sink,
) -> Result<(), Error> {
    let client = Client::new();
    let mut volume = sink.volume();
    loop {
        let input = rx.recv().await.context("Audio channel closed")?;
        match input.as_str() {
            "volume" => {
                // internal event
                let received_volume = volume_rx
                    .recv()
                    .await // this shouldn't stall the audio loop since volume_rx should already have a message by the time volume is received
                    .context("Received volume from keybind listener was not of type f32")?; // this shouldnt ever happen
                sink.set_volume(received_volume);
                volume = received_volume;
                update_audio_status(volume, "Changed volume");
            }
            "died" => {
                sink.set_volume(volume / 2.0);
                update_audio_status(
                    volume / 2.0,
                    &format!("Player died, setting volume to {}", volume * 100.0 / 2.0),
                );
            }
            "left" => {
                sink.stop();
                update_audio_status(volume, "Player left the game, stopping audio output");
            }
            _ => {
                sink.set_volume(volume);
                play_audio(&input, &client, &sink).await?;
                update_audio_status(volume, &format!("Currently playing URL {input}"));
            }
        }
    }
}

async fn play_audio(url: &str, client: &Client, sink: &Sink) -> Result<(), Error> {
    println!("Got request to play audio {url}");

    sink.stop();

    let cache_file = fs::read("fe2io-cache/cache.json").await?;
    let mut cache: AudioCache;
    if cache_file.is_empty() {
        let json: Value = json!({
            "urls": [],
            "files": [],
        });
        cache = serde_json::from_value(json)?;
    } else {
        cache = serde_json::from_slice(&cache_file)?;
    }

    let filename = generate(32, CHARSET);
    let file_as_str = format!("fe2io-cache/{filename}");
    let mut file = PathBuf::new();
    file.set_file_name(file_as_str);

    let mut file_exists = false;
    for (i, cache_url) in cache.urls.iter().enumerate() {
        if cache_url == url {
            let file_as_str = &cache.files[i];
            file.set_file_name(file_as_str);
            file_exists = true;
            break;
        }
    }

    let start = Instant::now(); // Get the Instant before downloading or reading the audio

    let audio = if file_exists {
        let buf = fs::read(&file).await?;
        Cursor::new(buf)
    } else {
        // Get response from URL
        let response = client.get(url).send().await?.bytes().await?.to_vec(); // convert to vec so that the arms are the same types
        
        println!("Got response");

        let mut f = File::create(&file).await?;
        f.write_all(&response).await?;

        cache.urls.push(url.to_owned());
        cache.files.push(filename);

        let mut cache_file = fs::OpenOptions::new().write(true).truncate(true).open("fe2io-cache/cache.json").await?;
        cache_file.write_all(serde_json::to_string(&cache)?.as_bytes()).await?;
        // Wrap the response in a Cursor to implement Seek and Read 
        Cursor::new(response)
    };
    // Play the downloaded audio
    let decoder = Decoder::new(audio)?; 
    let elapsed = Instant::now().duration_since(start); // Get the Instant after downloading the audio, then convert it to a Duration representing the time since before the audio was downloaded
    sink.append(decoder.skip_duration(elapsed)); // Append decoder to sink and skip the elapsed Duration
    println!("Playback started");
    Ok(())
}

fn update_audio_status(volume: f32, status: &str) {
    #[cfg(not(debug_assertions))] // Only clear screen in case debug is disabled
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char); // clear screen in release builds
    println!("Status: {status}");
    println!("Volume: {}", volume * 100.0);
}
