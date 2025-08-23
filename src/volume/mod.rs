use std::fmt::{self, Display};
use std::process::Command;

pub enum VolumeChange {
    Up(i32),
    Down(i32),
}

#[derive(Debug, Clone, Copy)]
pub struct Percentage(pub i32);

impl Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

trait VolumeSetter {
    fn change_volume(&self, change: VolumeChange) -> Result<(), Box<dyn std::error::Error>>;
    fn set_volume(&self, new_volume: Percentage) -> Result<(), Box<dyn std::error::Error>>;
}

struct DefaultSetter {}

impl VolumeSetter for DefaultSetter {
    fn change_volume(&self, change: VolumeChange) -> Result<(), Box<dyn std::error::Error>> {
        let formatted = match change {
            VolumeChange::Up(i) => format!("+{i}%"),
            VolumeChange::Down(i) => format!("-{i}%"),
        };

        Self::set(formatted.as_str())?;

        Ok(())
    }

    fn set_volume(&self, new_volume: Percentage) -> Result<(), Box<dyn std::error::Error>> {
        let new_volume = format!("{new_volume}%");

        Self::set(new_volume.as_str())?;

        Ok(())
    }
}

impl DefaultSetter {
    fn set(change: &str) -> Result<(), Box<dyn std::error::Error>> {
        Command::new("pactl")
            .args(["set-sink-volume", "@DEFAULT_SINK@", change])
            .status()?;

        Ok(())
    }
}
