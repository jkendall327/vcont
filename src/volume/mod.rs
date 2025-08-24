mod percentage;
mod ramp;
mod setter;

pub use percentage::{Percentage, PercentageError};
pub use setter::VolumeSetter;

use crate::volume::setter::DefaultSetter;

pub fn system_volume() -> impl VolumeSetter {
    DefaultSetter {}
}
