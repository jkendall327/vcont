use crate::volume::{VolumeChange, VolumeSetter};

mod volume;

use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        println!("Hello, world!");

        let changer = volume::system_volume();

        let percentage = 65.try_into()?;

        changer.change_volume(VolumeChange::Up(percentage))?;

        sleep(Duration::from_secs(5 * 60)).await;
    }
}
