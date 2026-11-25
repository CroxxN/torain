use bencode::error;
use bencode::utils;
use std::array::TryFromSliceError;
use std::{fmt::Display, time::SystemTimeError};
use uttd::error::UrlError;
use uttd::UttdError;

#[derive(Debug, Clone)]
pub enum DHTError {
    // DHT Query errors
    GenericError,
    ServerError,
    ProtocError,
    UnknownMethod,
    // other errors
    FailedSystemTimeGen(SystemTimeError),
    UrlFormError(UrlError),
    RequestError,
    Serde(SerdeError),
}

impl DHTError {
    pub fn error_code(&self) -> u32 {
        match self {
            DHTError::GenericError => 201,
            DHTError::ServerError => 202,
            DHTError::ProtocError => 203,
            DHTError::UnknownMethod => 204,
            _ => 0,
        }
    }
    pub fn error_message(&self) -> String {
        match self {
            DHTError::GenericError => "A Generic Error Occured".into(),
            DHTError::ServerError => "A Server Error Occured".into(),
            DHTError::ProtocError => "Malformed Packet Detected".into(),
            DHTError::UnknownMethod => "Unknown argument or method".into(),
            _ => "Unkonwn error occured".into(),
        }
    }
}

impl PartialEq for DHTError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::GenericError, Self::GenericError) => true,
            (Self::ServerError, Self::ServerError) => true,
            (Self::ProtocError, Self::ProtocError) => true,
            (Self::UnknownMethod, Self::UnknownMethod) => true,
            (Self::FailedSystemTimeGen(a), Self::FailedSystemTimeGen(b)) => {
                a.duration() == b.duration()
            }
            _ => false,
        }
    }
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
            DHTError::UrlFormError(u) => write!(f, "Error forming url: {u}"),
            DHTError::Serde(s) => write!(f, "Serde Error: {s}"),
            DHTError::RequestError => write!(f, "Failed to send or receive packets"),
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

impl From<UrlError> for DHTError {
    fn from(value: UrlError) -> Self {
        Self::UrlFormError(value)
    }
}

impl From<UttdError> for DHTError {
    fn from(_value: UttdError) -> Self {
        Self::RequestError
    }
}

impl From<SerdeError> for DHTError {
    fn from(value: SerdeError) -> Self {
        Self::Serde(value)
    }
}

#[derive(Debug, Clone)]
pub enum SerdeError {
    ParsingBencode(error::DecodeError),
    KeyError(utils::BencodeErr),
    BencodeGenericError,
    UnknownDHTError,
    NoValidIDPresent,
    InvalidMessageType,
    MissingKey(Keys),
    NoHintProvided,
    SliceConversion(TryFromSliceError),
    ParsingUrl(UrlError),
    VecSlice,
    // TODO: Remove
    UnimplementedQueryParsing,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Keys {
    TransactionId,
    MessageType,
    Response,
    Query,
    QueryArguments,
    Id,
    Target,
    Token,
    InfoHash,
    Values,
    ImpliedPort,
    Port,
    Node,
    Error,
}
impl PartialEq for SerdeError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ParsingBencode(a), Self::ParsingBencode(b)) => a == b,
            (Self::KeyError(a), Self::KeyError(b)) => a == b,
            (Self::BencodeGenericError, Self::BencodeGenericError) => true,
            (Self::UnknownDHTError, Self::UnknownDHTError) => true,
            (Self::NoValidIDPresent, Self::NoValidIDPresent) => true,
            (Self::InvalidMessageType, Self::InvalidMessageType) => true,
            (Self::MissingKey(a), Self::MissingKey(b)) => a == b,
            (Self::NoHintProvided, Self::NoHintProvided) => true,
            (Self::SliceConversion(_), Self::SliceConversion(_)) => true,
            (Self::ParsingUrl(a), Self::ParsingUrl(b)) => a == b,
            (Self::VecSlice, Self::VecSlice) => true,
            (Self::UnimplementedQueryParsing, Self::UnimplementedQueryParsing) => true,
            _ => false,
        }
    }
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
            Self::NoHintProvided => write!(f, "No deocde hint provided"),
            Self::SliceConversion(x) => write!(f, "Failed to convert slice: {x}"),
            Self::ParsingUrl(u) => write!(f, "Failed to parse url: {u}"),
            Self::VecSlice => write!(f, "Failed to convert vec to slice"),
            Self::MissingKey(k) => write!(f, "{k}"),
        }
    }
}

impl Display for Keys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TransactionId => write!(f, "Missing Key 'tt'"),
            Self::MessageType => write!(f, "Missing key 'y'"),
            Self::Response => write!(f, "Missing key 'r'"),
            Self::Query => write!(f, "Missing key 'q'"),
            Self::QueryArguments => write!(f, "Missing key 'a'"),
            Self::Id => write!(f, "Missing key 'id'"),
            Self::Target => write!(f, "Missing key 'target'"),
            Self::Token => write!(f, "Missing key 'token'"),
            Self::InfoHash => write!(f, "Missing key 'info_hash'"),
            Self::ImpliedPort => write!(f, "Missing key 'implied_port"),
            Self::Port => write!(f, "Missing key 'port"),
            Self::Values => write!(f, "Missing key 'values'"),
            Self::Node => write!(f, "Missing key 'node'"),
            Self::Error => write!(f, "Missing key 'e'"),
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

impl From<TryFromSliceError> for SerdeError {
    fn from(value: TryFromSliceError) -> Self {
        Self::SliceConversion(value)
    }
}

impl From<UrlError> for SerdeError {
    fn from(value: UrlError) -> Self {
        Self::ParsingUrl(value)
    }
}

impl From<Vec<u8>> for SerdeError {
    fn from(_value: Vec<u8>) -> Self {
        Self::VecSlice
    }
}

#[derive(Debug)]
pub enum D2H2ClientError {
    MoveOutofArcError,
    UrlFormError(UrlError),
}

impl Display for D2H2ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            D2H2ClientError::MoveOutofArcError => {
                write!(f, "Error: Breaking Error: Moving out of Arc")
            }
            D2H2ClientError::UrlFormError(e) => write!(f, "Invalid Url: {e}"),
        }
    }
}

impl From<UrlError> for D2H2ClientError {
    fn from(value: UrlError) -> Self {
        Self::UrlFormError(value)
    }
}
