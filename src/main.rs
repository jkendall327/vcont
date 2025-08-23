mod volume;

use volume::{VolumeChange, VolumeSetter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let changer = volume::system_volume();

    let percentage = volume::Percentage::new(65).expect("Invalid percentage");

    changer.change_volume(VolumeChange::Up(percentage))?;

    Ok(())
}
