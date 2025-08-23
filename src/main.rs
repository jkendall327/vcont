use crate::volume::{VolumeChange, VolumeSetter};

mod volume;

use nonempty::{NonEmpty, nonempty};
use tokio::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let awakenings = build_schedule();
    let mut idx = 0;

    loop {
        println!("Invoking...!");

        let changer = volume::system_volume();

        let percentage = 65.try_into()?;

        changer.change_volume(VolumeChange::Up(percentage))?;

        let current: Instant = awakenings[idx];

        tokio::time::sleep_until(current).await;

        idx = (idx + 1) % awakenings.len();
    }
}

fn build_schedule() -> NonEmpty<Instant> {
    nonempty![Instant::from_std(std::time::Instant::now())]
}
