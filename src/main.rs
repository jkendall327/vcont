use crate::volume::{Percentage, VolumeChange, VolumeSetter};

mod volume;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let changer = volume::system_volume();

    let percentage = Percentage::new(65).expect("Invalid percentage");

    changer.change_volume(VolumeChange::Up(percentage))?;

    Ok(())
}
