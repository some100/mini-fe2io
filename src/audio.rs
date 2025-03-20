use anyhow::{Context, Error, bail};
use reqwest::Client;
use rodio::{Decoder, Sink, Source};
use std::{
    io::Cursor, 
    path::Path,
};
use tokio::{
    fs::{File, create_dir_all},
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc::Receiver,
    time::Instant,
};

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
    // Stop the current playback, if any
    sink.stop();

    // Get the user's home directory, and create the fe2io-cache directory
    let home = dirs::home_dir() // get home dir
        .context("No home directory was found")? // unwrap Option from home dir
        .to_str() // convert returned PathBuf to str
        .context("Home directory that was returned wasn't a string")? // unwrap Option from str
        .to_owned(); // convert str to owned String

    let cache = format!("{home}/fe2io-cache"); // get fe2io-cache as a string
    create_dir_all(cache.clone()).await?; // if fe2io-cache doesn't exist, create it

    let filtered_url = url.replace(&['/','<','>',':','\"','\\','|','?','*'][..], ""); // replace all "unprintable characters," specifically / < > : " \ | ? * with nothing
    let file_as_str = format!("{cache}/{filtered_url}"); // get file location as a string
    let file = Path::new(&file_as_str); // get file location as a Path

    let start = Instant::now(); // Get the Instant before downloading or reading the audio

    let audio = if let Ok(false) = file.try_exists() {
        // checks if file does not exist
        // Get response from URL
        let response = client.get(url).send().await?.bytes().await?.to_vec(); // convert to vec so that the arms are the same types
        
        println!("Got response");

        // Create a file inside of fe2io-cache, then write the content of Response to it
        let mut f = File::create(&file).await?;
        f.write_all(&response).await?;

        // Wrap the response in a Cursor to implement Seek and Read
        Cursor::new(response)
    } else if let Ok(true) = file.try_exists() {
        // checks if file does exist and is readable
        let mut f = File::open(file).await?;
        let mut buf = Vec::new();

        f.read_to_end(&mut buf).await?;
        Cursor::new(buf)
    } else {
        // this is probably because there are no permissions to read from home directory
        bail!("Unable to verify if fe2io-cache dir in home directory exists");
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
