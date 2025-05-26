use std::{fmt::Display, time::SystemTimeError};

#[derive(Debug)]
pub enum DHTError {
    // DHT Query errors
    GenericError,
    ServerError,
    ProtocError,
    UnknownMethod,

    // other errors
    FailedSystemTimeGen(SystemTimeError),
}

impl Display for DHTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DHTError::GenericError => write!(f, "Error: 201 A Generic Error Occured"),
            DHTError::ServerError => write!(f, "Error: 202 A Server Error Occured"),
            DHTError::ProtocError => write!(
                f,
                "Error: 203 Protocol Error: Malformed Packet or Invalid Arguments"
            ),
            DHTError::UnknownMethod => write!(f, "Error: 204 Protocol Error: Unknown Method"),
            DHTError::FailedSystemTimeGen(e) => {
                write!(f, "Error: Failed to generate system time: {}", e)
            }
        }
    }
}

impl From<SystemTimeError> for DHTError {
    fn from(value: SystemTimeError) -> Self {
        Self::FailedSystemTimeGen(value)
    }
}
