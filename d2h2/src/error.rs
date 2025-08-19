use bencode::error;
use bencode::utils;
use std::{fmt::Display, time::SystemTimeError};
use uttd::error::UrlError;

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

#[derive(Debug)]
pub enum SerdeError {
    ParsingBencode(error::DecodeError),
    KeyError(utils::BencodeErr),
    BencodeGenericError,
    UnknownDHTError,
}

impl From<error::DecodeError> for SerdeError {
    fn from(value: error::DecodeError) -> Self {
        Self::ParsingBencode(value)
    }
}

impl From<bencode::utils::BencodeErr> for SerdeError {
    fn from(value: bencode::utils::BencodeErr) -> Self {
        Self::KeyError(value)
    }
}

#[derive(Debug)]
pub enum D2H2ClientError {
    MoveOutofArcError,
    UrlFormError(UrlError),
}

impl From<UrlError> for D2H2ClientError {
    fn from(value: UrlError) -> Self {
        Self::UrlFormError(value)
    }
}
