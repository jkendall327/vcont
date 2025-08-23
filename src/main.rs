use crate::volume::{VolumeChange, VolumeSetter};

mod volume;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let changer = volume::system_volume();

    let percentage = 65.try_into()?;

    changer.change_volume(VolumeChange::Up(percentage))?;

    Ok(())
}
