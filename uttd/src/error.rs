use std::num::ParseIntError;

#[derive(Debug)]
pub enum UrlError {
    InvalidUrl,
    ParseIntError(std::num::ParseIntError),
}

impl From<ParseIntError> for UrlError {
    fn from(value: ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}
