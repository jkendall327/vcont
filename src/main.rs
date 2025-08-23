mod volume;

use volume::{VolumeChange, VolumeSetter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let changer = volume::system_volume();

    changer.change_volume(VolumeChange::Up(65))?;

    Ok(())
}
