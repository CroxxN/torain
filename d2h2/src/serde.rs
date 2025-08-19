#![allow(dead_code, unused_variables)]

use crate::error::{self, DHTError};
use bencode::{bencode, BTypes};

#[derive(Debug)]
pub struct KRPC {
    pub transaction_id: String,
    pub message_type: MessageType,
}

#[derive(Debug)]
pub enum MessageType {
    Query(Query),
    Response(BTypes),
    Error(error::DHTError),
}

#[derive(Debug)]
pub struct Query {
    method_name: String,
    // A dictionary of btypes for named argument pairs. Useful for later serialization
    // Provide a utility function to parse these arguments
    arguments: BTypes,
}

// INFO: de
pub fn deserialize_dht(packet: Vec<u8>) -> Result<KRPC, error::SerdeError> {
    let deserialized = if let BTypes::DICT(d) = bencode::decode(&mut packet.into_iter())? {
        d
    } else {
        // TODO: fix this
        return Err(error::SerdeError::BencodeGenericError);
    };
    let transaction_id: String = deserialized.get("t").unwrap().try_into()?;
    let y: String = deserialized.get("y").unwrap().try_into()?;

    // if key 'y' has a value of e, we encountered an error
    if &y == "e" {
        let dht_error;
        let error: &BTypes = deserialized.get("e").unwrap();
        if let &BTypes::LIST(l) = &error {
            if let BTypes::INT(bint) = l[0] {
                match bint {
                    201 => dht_error = DHTError::GenericError,
                    202 => dht_error = DHTError::ServerError,
                    203 => dht_error = DHTError::ProtocError,
                    _ => dht_error = DHTError::UnknownMethod,
                }
                return Ok(KRPC {
                    transaction_id,
                    message_type: MessageType::Error(dht_error),
                });
            }
        }
        return Err(error::SerdeError::UnknownDHTError);
    }

    if &y == "r" {
        if let BTypes::DICT(d) = deserialized.get("r").unwrap() {
            // TODO: decode the response message. Also think about how we wanna give this back to the
            // overall DHT implementation---as it's used by both the client and the server.
            // We may need a few helper functions to deal with the type (4) of the available KPRC.
            todo!()
        }
    }

    if &y == "q" {
        let method_name: String = deserialized.get("q").unwrap().try_into()?;
        if let BTypes::DICT(d) = deserialized.get("q").unwrap() {
            todo!()
        }
        todo!()
    }

    // CONTINUE:

    // TODO:
    todo!()
}
