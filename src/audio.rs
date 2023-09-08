// src/audio.rs

extern crate reqwest;
extern crate tempfile;
extern crate rodio;
extern  crate uuid;

use std::io::BufReader;
use std::io::Write;
use std::fs::File;
use tempfile::tempdir;
use rodio::{Decoder, OutputStream, Sink};
use reqwest::Client;
use uuid::Uuid;

pub struct AudioPlayer {
    sink: Sink,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().expect("Failed to create audio stream");
        let sink = Sink::try_new(&stream_handle).expect("Failed to create audio sink");
        Self { sink }
    }

    pub async fn play_audio(&self, url: &str) {
        println!("Balling");
        // Stop the current playback, if any
        self.sink.stop();
        // Create a temporary directory
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // Generate a unique file name
    let file_extension = url.split('.').last().unwrap();
    let file_name = Uuid::new_v4().to_string() + "." + file_extension;

    // Create a reqwest client
    let client = Client::new();

    // Download the file asynchronously
    let response = client.get(url).send().await.expect("Failed to send request");

    // Create a file to save the downloaded content
    let file_path = temp_dir.path().join(&file_name);
    let mut file = File::create(&file_path).expect("Failed to create file");

    // Save the downloaded content to the file
    file.write_all(&response.bytes().await.expect("Failed to read response body")).expect("Failed to save file");

    println!("File saved to: {:?}", file_path);

    // Play the downloaded audio
    let file = File::open(file_path).expect("Failed to open audio file");
    let decoder = Decoder::new(BufReader::new(file)).expect("Failed to decode audio file");
    self.sink.append(decoder);

    // Start playback
    self.sink.play();
    println!("Playback started");
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }

    pub fn stop(&self) {
        self.sink.stop();
    }
}
