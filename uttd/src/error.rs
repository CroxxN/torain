use std::{net::AddrParseError, num::ParseIntError};

#[derive(Debug)]
pub enum UrlError {
    InvalidUrl,
    ParseIntError(std::num::ParseIntError),
    AddressParseError,
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
