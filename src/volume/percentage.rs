use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy)]
pub struct Percentage(u8);

impl Percentage {
    pub fn new(value: u8) -> Option<Self> {
        match value {
            v if v <= 100 => Some(Self(value)),
            _ => None,
        }
    }

    pub fn value(self) -> u8 {
        self.0
    }
}

impl Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
