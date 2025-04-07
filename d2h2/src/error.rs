use std::fmt::Display;

#[derive(Debug)]
pub enum DHTError {
    Error(u32),
}

impl Display for DHTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DHTError::Error(e) => write!(f, "{}", *e),
        }
    }
}
