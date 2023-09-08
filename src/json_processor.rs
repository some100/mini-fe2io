extern crate serde;
extern crate serde_json;

use super::audio::AudioPlayer;

use serde::{Deserialize, Serialize};

// Message Format
#[derive(Default, Debug, Serialize, Deserialize)]
struct MessageFormat {
    msgType: String,
    statusType: Option<String>,
    audioUrl: Option<String>,
}

pub async fn process_data(data: &str, player: &AudioPlayer) {
    let p_data: MessageFormat = match serde_json::from_str(data) {
        Ok(parsed_data) => parsed_data,
        Err(err) => {
            eprintln!("Error parsing JSON: {}", err);
            return; // Exit early on error
        }
    };

    match p_data.msgType.as_str() {
        "bgm" => {
            if let Some(audio_url) = &p_data.audioUrl 
            {
                player.play_audio(audio_url).await;
            }
        },
        "gameStatus" => {
            match p_data.statusType.as_deref() {
                Some("died") => {
                    player.set_volume(0.5);
                },
                Some("left") => {
                    player.stop();
                },
                _ => {} // Handle other status types as needed
            }
        },
        _ => {
            // Handle unknown message types if necessary
        }
    }
}
