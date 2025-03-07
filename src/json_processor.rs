use tokio::sync::mpsc::Sender;
use serde::{Deserialize, Serialize};
use anyhow::{Context, Error};

// Message Format
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageFormat {
    msg_type: String,
    status_type: Option<String>,
    audio_url: Option<String>,
}

pub async fn process_data(data: &str, tx: &Sender<String>) -> Result<(), Error> {
    let p_data: MessageFormat = match serde_json::from_str(data) {
        Ok(parsed_data) => parsed_data,
        Err(err) => {
            eprintln!("Error parsing JSON: {err}");
            return Err(err.into()); // Exit early on error
        }
    };

    match p_data.msg_type.as_str() {
        "bgm" => get_audio(p_data, tx).await?,
        "gameStatus" => get_status(p_data, tx).await?,
        _ => eprintln!("No msgType was provided"),
    }
    Ok(())
}

async fn get_audio(p_data: MessageFormat, tx: &Sender<String>) -> Result<(), Error> {
    let audio_url = p_data.audio_url
        .context("msgType was bgm but no URL was provided")?;
    tx.send(audio_url).await?;
    Ok(())
}

async fn get_status(p_data: MessageFormat, tx: &Sender<String>) -> Result<(), Error> {
    let status_type = p_data.status_type
        .context("msgType was gameStatus but no status was provided")?;
    tx.send(status_type).await?;
    Ok(())
}