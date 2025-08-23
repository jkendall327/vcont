use crate::volume::{VolumeChange, VolumeSetter};

mod schedule;
mod volume;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut schedule = schedule::Schedule::new();

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
