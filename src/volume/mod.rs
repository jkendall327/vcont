use crate::volume::setter::*;

mod percentage;
mod setter;

pub use percentage::{Percentage, PercentageError};
pub use setter::{VolumeChange, VolumeSetter};

pub fn system_volume() -> impl VolumeSetter {
    DefaultSetter {}
}
