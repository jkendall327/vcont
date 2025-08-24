use regex::Regex;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use crate::{
    schedule::Invocation,
    volume::{PercentageError, percentage::Percentage, ramp},
};

// 20 updates per second
const RAMP_UPDATE_INTERVAL: tokio::time::Duration = tokio::time::Duration::from_millis(50);

static RE: OnceLock<Regex> = OnceLock::new();

#[derive(thiserror::Error, Debug)]
pub enum VolumeError {
    #[error("failed to spawn pactl: {0}")]
    Spawn(#[from] std::io::Error),
    #[error("pactl returned non-zero exit status: {status}. stderr: {stderr}")]
    Pactl { status: i32, stderr: String },
    #[error("parsing percentage failed")]
    Percentage(#[from] PercentageError),
    #[error("Setting the volume failed")]
    ThreadingError(#[from] tokio::task::JoinError),
}

type VolumeResult = Result<(), VolumeError>;

pub trait VolumeSetter {
    async fn process(&self, invocation: Invocation) -> VolumeResult;
}

pub struct DefaultSetter;

impl VolumeSetter for DefaultSetter {
    async fn process(&self, invocation: Invocation) -> VolumeResult {
        let now = std::time::Instant::now();

        let current_volume = get_volume().await?;

        let ramp = ramp::VolumeRamp::new(
            current_volume,
            invocation.desired_sound.value(),
            invocation.time,
            invocation.ramp_duration,
        );

        let mut last_set = None;

        loop {
            let now = std::time::Instant::now();

            let v: Percentage = ramp.value_at(now).try_into()?;

            // Avoid setting to current value unnecessarily.
            if last_set != Some(v) {
                let new_volume = format!("{v}%");

                set_async(new_volume.as_str()).await?;

                last_set = Some(v);
            }

            // We are done.
            if now >= invocation.time {
                // Set the final value one last time to be sure
                set_async(format!("{}%", invocation.desired_sound).as_str()).await?;
                return Ok(());
            }

            tokio::time::sleep(RAMP_UPDATE_INTERVAL).await;
        }

        Ok(())
    }
}

async fn set_async(change: &str) -> VolumeResult {
    let change = change.to_string();
    tokio::task::spawn_blocking(move || set(&change)).await?
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

async fn get_volume() -> Result<u8, VolumeError> {
    let output = tokio::task::spawn_blocking(|| {
        Command::new("pactl")
            .arg("get-sink-volume")
            .arg("@DEFAULT_SINK@")
            .output()
    })
    .await??; // The first '?' handles JoinError, the second handles io::Error

    if !output.status.success() {
        let status = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(VolumeError::Pactl { status, stderr });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let re = RE.get_or_init(|| Regex::new(r"(\d{1,3})%").unwrap());

    // Example output: "Volume: front-left: 32768 /  50% / -18.06 dB,   front-right: 32768 /  50% / -18.06 dB"
    if let Some(captures) = re.captures(&stdout) {
        // captures[1] will be the string of digits
        if let Ok(vol) = captures[1].parse::<u8>() {
            return Ok(vol);
        }
    }

    Err(VolumeError::Pactl {
        status: -1,
        stderr: format!("Failed to parse volume from pactl output: '{stdout}'"),
    })
}
