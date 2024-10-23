use crate::utils::BencodeErr;

#[derive(Debug)]
pub enum DecodeError {
    EOF,
    IntParseError(std::num::ParseIntError),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EOF => write!(f, "End of File Reached!"),
            Self::IntParseError(e) => write!(f, "Failed to parse integer: {e}"),
        }
    }
}

impl From<std::num::ParseIntError> for DecodeError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::IntParseError(value)
    }
}

impl From<BencodeErr> for DecodeError {
    fn from(_value: BencodeErr) -> Self {
        Self::EOF
    }
}
