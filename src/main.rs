#![allow(dead_code)]
#![allow(unused)]

use std::env;

use crate::{config::ScheduleItem, volume::VolumeSetter};

mod config;
mod schedule;
mod volume;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

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

    let config_file_path = args.get(1).cloned().unwrap_or("config.toml".to_owned());

    let config = config::load_config(config_file_path)
        .await
        .unwrap_or(config::AppConfig {
            ramp_duration_seconds: 60,
            schedule: default_schedule,
        });

    let mut schedule = schedule::Schedule::from_schedule_items(config.schedule)?;

    let mut next = schedule.get_next().ok_or("No schedule was established")?;

    tokio::time::sleep_until(next.get_start().into()).await;

    let changer = volume::system_volume();

    loop {
        println!("Invoking...!");

        changer.process(next).await;

        next = schedule
            .get_next()
            .expect("This method should always succeed if it has succeeded once before");

        tokio::time::sleep_until(next.get_start().into()).await;
    }
}
