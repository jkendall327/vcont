use std::process::Command;

use crate::volume::percentage::Percentage;

pub enum VolumeChange {
    Up(Percentage),
    Down(Percentage),
}

pub trait VolumeSetter {
    fn change_volume(&self, change: VolumeChange) -> Result<(), Box<dyn std::error::Error>>;
    fn set_volume(&self, new_volume: Percentage) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct DefaultSetter {}

impl VolumeSetter for DefaultSetter {
    fn change_volume(&self, change: VolumeChange) -> Result<(), Box<dyn std::error::Error>> {
        let formatted = match change {
            VolumeChange::Up(i) => format!("+{i}%"),
            VolumeChange::Down(i) => format!("-{i}%"),
        };

        set(formatted.as_str())?;

        Ok(())
    }

    fn set_volume(&self, new_volume: Percentage) -> Result<(), Box<dyn std::error::Error>> {
        let new_volume = format!("{new_volume}%");

        set(new_volume.as_str())?;

        Ok(())
    }
}

fn set(change: &str) -> Result<(), Box<dyn std::error::Error>> {
    Command::new("pactl")
        .args(["set-sink-volume", "@DEFAULT_SINK@", change])
        .status()?;

    Ok(())
}
