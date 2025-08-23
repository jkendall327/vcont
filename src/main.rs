use crate::volume::{VolumeChange, VolumeSetter};

mod volume;

use tokio::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        println!("Invoking...!");

        let changer = volume::system_volume();

        let percentage = 65.try_into()?;

        changer.change_volume(VolumeChange::Up(percentage))?;

        let wake_time = Instant::now() + Duration::from_secs(5 * 60);

        tokio::time::sleep_until(wake_time).await;
    }
}
