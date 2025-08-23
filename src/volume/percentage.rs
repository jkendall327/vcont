use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy)]
pub struct Percentage(u8);

#[derive(thiserror::Error, Debug)]
pub enum PercentageError {
    #[error("percentage {value} is out of range ({min}..={max} allowed)")]
    OutOfRange { value: u8, min: u8, max: u8 },
}

impl Percentage {
    const MIN: u8 = 0;
    const MAX: u8 = 100;

    fn validate(value: u8) -> Result<(), PercentageError> {
        if value <= Self::MAX {
            Ok(())
        } else {
            Err(PercentageError::OutOfRange {
                value,
                min: Self::MIN,
                max: Self::MAX,
            })
        }
    }

    pub fn new(value: u8) -> Result<Self, PercentageError> {
        Self::validate(value)?;
        Ok(Self(value))
    }

    pub fn value(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for Percentage {
    type Error = PercentageError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl std::str::FromStr for Percentage {
    type Err = PercentageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u8 = s.trim().parse().map_err(|_| {
            PercentageError::OutOfRange {
                value: u8::MAX, // sentinel value since parsing failed
                min: Self::MIN,
                max: Self::MAX,
            }
        })?;
        Self::new(value)
    }
}

impl Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
