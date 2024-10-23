use std::fmt::Display;

#[derive(Debug)]
pub enum TorrentError {
    UnexpectedField,
}

impl Display for TorrentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unexpected field encountered. Aborting")
    }
}
