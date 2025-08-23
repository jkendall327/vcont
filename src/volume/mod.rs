use std::process::Command;

pub enum VolumeChange {
    Up(i32),
    Down(i32),
}

#[derive(Debug)]
pub struct Percentage(i32);

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
        let new_volume = format!("{new_volume:#?}%");

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
