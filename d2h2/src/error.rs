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
    NoValidIDPresent,
    InvalidMessageType,
    // TODO: Remove
    UnimplementedQueryParsing,
}

impl Display for SerdeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParsingBencode(e) => write!(f, "Error Parsing Bencode: {e}"),
            Self::KeyError(e) => write!(f, "Key error: {e}"),
            Self::BencodeGenericError => write!(f, "DHT Serde: Generic Error Parsing Bencode"),
            Self::UnknownDHTError => write!(f, "Error: Unknown DHT Error Encountered"),
            Self::NoValidIDPresent => write!(f, "Error: No valid id present on the packet"),
            Self::InvalidMessageType => write!(f, "Error: Invalid message type"),
            // TODO: Remove
            Self::UnimplementedQueryParsing => write!(
                f,
                "Warning: Implemented DHT message parsing. You should NOT have reached this"
            ),
        }
    }
}

impl From<error::DecodeError> for SerdeError {
    fn from(value: error::DecodeError) -> Self {
        Self::ParsingBencode(value)
    }
}

impl From<utils::BencodeErr> for SerdeError {
    fn from(value: utils::BencodeErr) -> Self {
        Self::KeyError(value)
    }
}

#[derive(Debug)]
pub enum D2H2ClientError {
    MoveOutofArcError,
    UrlFormError(UrlError),
    NetworkError(uttd::UttdError),
    Serde(SerdeError),
    DHT(DHTError),
    LookupTimeout,
}

impl Display for D2H2ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            D2H2ClientError::MoveOutofArcError => {
                write!(f, "Error: Breaking Error: Moving out of Arc")
            }
            D2H2ClientError::UrlFormError(e) => write!(f, "Invalid Url: {e}"),
            D2H2ClientError::NetworkError(e) => write!(f, "Network Error: {e:?}"),
            D2H2ClientError::Serde(e) => write!(f, "Serde Error: {e}"),
            D2H2ClientError::DHT(e) => write!(f, "DHT Error: {e}"),
            D2H2ClientError::LookupTimeout => write!(f, "Error: DHT lookup timed out"),
        }
    }
}

impl From<UrlError> for D2H2ClientError {
    fn from(value: UrlError) -> Self {
        Self::UrlFormError(value)
    }
}

impl From<uttd::UttdError> for D2H2ClientError {
    fn from(value: uttd::UttdError) -> Self {
        Self::NetworkError(value)
    }
}

impl From<SerdeError> for D2H2ClientError {
    fn from(value: SerdeError) -> Self {
        Self::Serde(value)
    }
}

impl From<DHTError> for D2H2ClientError {
    fn from(value: DHTError) -> Self {
        Self::DHT(value)
    }
}
