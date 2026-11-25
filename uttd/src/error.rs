use std::{fmt::Display, net::AddrParseError, num::ParseIntError, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub enum UrlError {
    InvalidUrl,
    ParseIntError(std::num::ParseIntError),
    AddressParseError,
    NumberConversionError(<u8 as FromStr>::Err),
}

impl Display for UrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UrlError::InvalidUrl => write!(f, "Invalid Url Encountered"),
            UrlError::ParseIntError(e) => write!(f, "Error Parsing Integer in URL: {e}"),
            UrlError::AddressParseError => write!(f, "Failed to parse URL"),
            UrlError::NumberConversionError(x) => write!(f, "Failed to parse diits: {}", x),
        }
    }
}

impl From<ParseIntError> for UrlError {
    fn from(value: ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}

impl From<AddrParseError> for UrlError {
    fn from(_: AddrParseError) -> Self {
        Self::AddressParseError
    }
}
