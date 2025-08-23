use std::process::{Command, Stdio};

use crate::{
    schedule::Invocation,
    volume::{PercentageError, percentage::Percentage, ramp},
};

#[derive(thiserror::Error, Debug)]
pub enum VolumeError {
    #[error("failed to spawn pactl: {0}")]
    Spawn(#[from] std::io::Error),
    #[error("pactl returned non-zero exit status: {status}. stderr: {stderr}")]
    Pactl { status: i32, stderr: String },
    #[error("parsing percentage failed")]
    Percentage(#[from] PercentageError),
}

pub enum VolumeChange {
    Up(Percentage),
    Down(Percentage),
}

type VolumeResult = Result<(), VolumeError>;

pub trait VolumeSetter {
    fn process(&self, invocation: Invocation) -> impl Future<Output = VolumeResult>;
}

pub struct DefaultSetter {}

impl DefaultSetter {
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

impl VolumeSetter for DefaultSetter {
    async fn process(&self, invocation: Invocation) -> VolumeResult {
        let now = std::time::Instant::now();
        let end = invocation.time;

        let duration = end - now;

        let current_volume = get_volume().unwrap();

        let ramp = ramp::VolumeRamp::new(
            current_volume,
            invocation.desired_sound.value(),
            end,
            duration,
        );

        let mut last_set: Option<Percentage> = None;

        loop {
            let now = std::time::Instant::now();

            let v: Percentage = ramp.value_at(now).try_into()?;

            // Avoid setting to current value unnecessarily.
            if last_set != Some(v) {
                self.set_volume(v);
                last_set = Some(v);
            }

            // We are done.
            if let Some(s) = last_set
                && s == invocation.desired_sound
            {
                return Ok(());
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

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

fn get_volume() -> Option<u8> {
    let output = Command::new("pactl")
        .arg("get-sink-volume")
        .arg("@DEFAULT_SINK@")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Example output: "Volume: front-left: 32768 /  50% / -18.06 dB,   front-right: 32768 /  50% / -18.06 dB"
    for part in stdout.split_whitespace() {
        if part.ends_with('%') {
            if let Ok(vol) = part.trim_end_matches('%').parse::<u8>() {
                return Some(vol);
            }
        }
    }

    None
}
