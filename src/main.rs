use crate::volume::{VolumeChange, VolumeSetter};

mod schedule;
mod volume;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let awakenings = schedule::build_schedule();
    let mut idx = 0;

    loop {
        println!("Invoking...!");

        let changer = volume::system_volume();

        let percentage = 65.try_into()?;

        changer.change_volume(VolumeChange::Up(percentage))?;

        let current = awakenings[idx];

        tokio::time::sleep_until(current.awakening).await;

        idx = (idx + 1) % awakenings.len();
    }
}
