use std::process::{Command, Stdio};

use crate::volume::percentage::Percentage;

#[derive(thiserror::Error, Debug)]
pub enum VolumeError {
    #[error("failed to spawn pactl: {0}")]
    Spawn(#[from] std::io::Error),
    #[error("pactl returned non-zero exit status: {status}. stderr: {stderr}")]
    Pactl { status: i32, stderr: String },
}

pub enum VolumeChange {
    Up(Percentage),
    Down(Percentage),
}

type VolumeResult = Result<(), VolumeError>;

pub trait VolumeSetter {
    fn change_volume(&self, change: VolumeChange) -> VolumeResult;
    fn set_volume(&self, new_volume: Percentage) -> VolumeResult;
}

pub struct DefaultSetter {}

impl VolumeSetter for DefaultSetter {
    fn change_volume(&self, change: VolumeChange) -> VolumeResult {
        let formatted = match change {
            VolumeChange::Up(i) => format!("+{i}%"),
            VolumeChange::Down(i) => format!("-{i}%"),
        };

        set(formatted.as_str())?;

        Ok(())
    }

    fn set_volume(&self, new_volume: Percentage) -> VolumeResult {
        let new_volume = format!("{new_volume}%");

        set(new_volume.as_str())?;

        Ok(())
    }
}

fn set(change: &str) -> VolumeResult {
    let output = Command::new("pactl")
        .args(["set-sink-volume", "@DEFAULT_SINK@", change])
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        let status = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(VolumeError::Pactl { status, stderr });
    }

    Ok(())
}
