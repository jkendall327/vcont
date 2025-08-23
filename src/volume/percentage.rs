use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Percentage(u8);

#[derive(thiserror::Error, Debug)]
pub enum PercentageError {
    #[error("percentage {value} is out of range ({min}..={max} allowed)")]
    OutOfRange { value: u8, min: u8, max: u8 },
    #[error("'{invalid_str}' is not a valid number for a percentage")]
    ParseFailed { invalid_str: String },
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
        let s_trimmed = s.trim();

        let value: u8 = s_trimmed
            .parse()
            .map_err(|_| PercentageError::ParseFailed {
                invalid_str: s_trimmed.to_string(),
            })?;

        Self::new(value)
    }
}

impl Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
