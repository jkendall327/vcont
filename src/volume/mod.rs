use std::fmt::{self, Display};
use std::process::Command;

pub enum VolumeChange {
    Up(Percentage),
    Down(Percentage),
}

#[derive(Debug, Clone, Copy)]
pub struct Percentage(u8);

impl Percentage {
    pub fn new(value: u8) -> Option<Self> {
        match value {
            v if v <= 100 => Some(Self(value)),
            _ => None,
        }
    }

    pub fn value(self) -> u8 {
        self.0
    }
}

impl Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait VolumeSetter {
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

pub fn system_volume() -> impl VolumeSetter {
    DefaultSetter {}
}
