#![allow(dead_code)]
#![allow(unused)]

use std::env;

use tokio::signal;
use tracing::{debug, info, warn};

use crate::{
    config::ScheduleItem,
    schedule::{Invocation, Schedule, ScheduleError},
    volume::VolumeSetter,
};

mod config;
mod schedule;
mod volume;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_target(false).init();

    check_dependencies()?;

    info!("starting up");

    let mut schedule = get_schedule().await?;

    info!("built schedule");

    let mut next = schedule.get_next().ok_or("No schedule was established")?;

    wait_for_next(&next).await;

    let changer = volume::system_volume();

    let worker = async {
        loop {
            debug!("wait over, beginning work");

            changer.process(next).await;

            next = schedule
                .get_next()
                .expect("This method should always succeed if it has succeeded once before");

            wait_for_next(&next).await;
        }
    };

    tokio::select! {
        _ = worker => {}
        _ = signal::ctrl_c() => {
            info!("shutdown signal received");
        }
    }

    info!("clean exit");

    Ok(())
}

async fn wait_for_next(invocation: &Invocation) {
    tokio::time::sleep_until(invocation.get_start().into()).await;
}

async fn get_schedule() -> Result<Schedule, ScheduleError> {
    let args: Vec<String> = env::args().collect();

    let config_file_path = args.get(1).cloned().unwrap_or("config.toml".to_owned());

    let config = match config::load_config(&config_file_path).await {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!(
                "Failed to load config from '{}': {}. Using default schedule.",
                config_file_path, e
            );

            let default_schedule = vec![
                ScheduleItem {
                    time: "08:00".to_owned(),
                    volume: 54,
                },
                ScheduleItem {
                    time: "09:00".to_owned(),
                    volume: 23,
                },
            ];

            config::AppConfig {
                ramp_duration_seconds: 60,
                schedule: default_schedule,
            }
        }
    };

    schedule::Schedule::from_schedule_items(config.schedule, config.ramp_duration_seconds)
}

fn check_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    match which::which("pactl") {
        Ok(path) => {
            debug!("Found 'pactl' executable at: {}", path.display());
            Ok(())
        }
        Err(_) => {
            let error_message = "Dependency 'pactl' not found. \
            Please ensure PulseAudio (or a compatible provider like PipeWire) is installed \
            and that the 'pactl' command is available in your system's PATH.";

            eprintln!("Error: {error_message}");

            Err(error_message.into()) // Convert the string slice into a boxed error
        }
    }
}
