#![allow(dead_code)]
#![allow(unused)]

use crate::volume::{VolumeChange, VolumeSetter};

mod schedule;
mod volume;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw = vec![
        ("08:00".to_owned(), "54".to_owned()),
        ("09:00".to_owned(), "23".to_owned()),
    ];

    let mut schedule = schedule::Schedule::from_raw(raw)?;

    let next = schedule.get_next();

    tokio::time::sleep_until(next.time.into()).await;

    loop {
        println!("Invoking...!");

        let changer = volume::system_volume();

        changer.change_volume(VolumeChange::Up(next.desired_sound))?;

        let next = schedule.get_next();

        tokio::time::sleep_until(next.time.into()).await;
    }
}
