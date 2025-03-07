use tokio::{
    time::{sleep, Duration},
    sync::mpsc::Sender,
};
use device_query::{DeviceQuery, DeviceState, Keycode};
use anyhow::Error;

pub async fn keybind_listen(tx: Sender<String>, volume_tx: Sender<f32>, mut volume: f32) -> Result<(), Error> { // MASSIVE function
    let device_state = DeviceState::new();
    let mut initial_volume = volume;
    let mut muted = false;
    loop {
        let keys: Vec<Keycode> = device_state.get_keys();
        match keys {
            keys if keys.contains(&Keycode::Equal) => {
                volume = (((volume * 100.0) + 5.0).min(100.0).round()) / 100.0;
                volume_tx.send(volume).await?; // Increase volume
                tx.send("volume".to_owned()).await?; // Send volume event to audio loop
                muted = false;
            },
            keys if keys.contains(&Keycode::Minus) => {
                volume = (((volume * 100.0) - 5.0).max(0.0).round()) / 100.0;
                volume_tx.send(volume).await?; // Lower volume
                tx.send("volume".to_owned()).await?;
                muted = false;
            },
            keys if keys.contains(&Keycode::Grave) => { // this key `
                if muted {
                    volume = initial_volume;
                    volume_tx.send(volume).await?; // Unmute (sets volume to value before muted)
                    tx.send("volume".to_owned()).await?;
                    muted = false;
                } else {
                    initial_volume = volume; // Save initial volume in variable
                    volume = 0.0;
                    volume_tx.send(volume).await?; // Mute
                    tx.send("volume".to_owned()).await?;
                    muted = true;
                }
            },
            _ => (),
        }
        sleep(Duration::from_millis(100)).await; // sleep for 100 ms to avoid maxing out the cpu
    }
}