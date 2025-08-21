#![allow(dead_code, unused_variables)]

use crate::error::{self, DHTError};
use ::bencode::utils::vec_to_string;
use bencode::{bencode, BTypes};
use uttd::url::Url;

#[derive(Debug)]
pub struct KRPC {
    pub transaction_id: String,
    pub message_type: MessageType,
}

impl Default for KRPC {
    fn default() -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub enum MessageType {
    Query(Query),
    Response(Response),
    Error(error::DHTError),
}

#[derive(Debug)]
pub struct Response {
    // TODO: use &[u8] instead of String?
    id: String,
    // for find_node responses we don't get a token response type
    token: Option<String>,
    response: Option<ResponseType>,
}

#[derive(Debug)]
pub enum ResponseType {
    Node(Vec<Url>),
    Values(Vec<Url>),
}

#[derive(Debug)]
pub struct Query {
    method_name: String,
    // A dictionary of btypes for named argument pairs. Useful for later serialization
    // Provide a utility function to parse these arguments
    arguments: BTypes,
}

// INFO: de
// TODO: maybe add option disable unrelated packet processing for client mode for now?
pub fn deserialize_dht(packet: Vec<u8>) -> Result<KRPC, error::SerdeError> {
    let deserialized = if let BTypes::DICT(d) = bencode::decode(&mut packet.into_iter())? {
        d
    } else {
        // TODO: fix this
        return Err(error::SerdeError::BencodeGenericError);
    };
    let transaction_id: String = deserialized.get("t").unwrap().try_into()?;
    let y: String = deserialized.get("y").unwrap().try_into()?;

    // The key 'y' can have one of three values:
    //     a. 'e'
    //     b. 'q'
    //     c. 'r'
    // These three values lets the DHT node know WHICH type of packet this current one if.
    //     Key 'e' means that we've encountered an error. This is really only possible when
    //     we RECEIVED an response from another DHT node in client mode. So we don't really need
    //     to care about forming a packet with key 'e' when running in client mode. However,
    //     it is essential in server mode as sometimes we may encounter an error when trying to
    //     complete a request.
    //
    //     Key 'q' means that it is a query. In client mode, we can only form this. We'll NEVER
    //     handle this in client mode. However, in server mode, we will need to response appropriately
    //     to a query packet.
    //
    //     Key 'r' means that it is a response to a previous request we've made. If we get a reponse with
    //     key 'r', we can be pretty certain that we didn't encounter an error from the server (as had it been
    //     the case, we'd have encountered a packet with key 'e'). In client mode, we only RECEIVE this type
    //     of packet. In server mode, we will need to form packet with a key 'r'.

    // if we get an error
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
        let id: String;
        let mut token = None;
        if let BTypes::DICT(d) = deserialized.get("r").unwrap() {
            // TODO: decode the response message. Also think about how we wanna give this back to the
            // overall DHT implementation---as it's used by both the client and the server.
            // We may need a few helper functions to deal with the type (4) of the available KPRC.
            if let BTypes::BSTRING(bs) = d.get("id").unwrap() {
                id = vec_to_string(bs);
            } else {
                return Err(error::SerdeError::NoValidIDPresent);
            }
            if let BTypes::BSTRING(bs) = d.get("token").unwrap() {
                token = Some(vec_to_string(bs));
            }

            let mut resp = Response {
                id,
                token,
                response: None,
            };

            if let BTypes::BSTRING(bs) = d.get("nodes").unwrap() {
                let ips: Vec<Url> = bs
                    .chunks(6)
                    .map(|x| {
                        let ip: [u8; 4] = x[0..4].try_into().unwrap();
                        let port = u16::from_be_bytes(x[4..6].try_into().unwrap());
                        Url::from_ip_bytes(&ip, port)
                    })
                    .collect();
                resp.response = Some(ResponseType::Node(ips));
            } else if let BTypes::BSTRING(bs) = d.get("values").unwrap() {
                // create urls from the compact url info contained in bs
                // this containes the actual peers for the find_peers query
                let ips: Vec<Url> = bs
                    .chunks(6)
                    .map(|x| {
                        let ip: [u8; 4] = x[0..4].try_into().unwrap();
                        let port = u16::from_be_bytes(x[4..6].try_into().unwrap());
                        Url::from_ip_bytes(&ip, port)
                    })
                    .collect();
                resp.response = Some(ResponseType::Values(ips));
            }
            // TODO: change this to reflect the response we get
            return Ok(KRPC {
                transaction_id,
                message_type: MessageType::Response(resp),
            });
            // todo!()
        }
    }

    // TODO: we don't really care about this for now as we'll be mostly running in client mode
    if &y == "q" {
        // TODO: Remove
        // let method_name: String = deserialized.get("q").unwrap().try_into()?;
        // if let BTypes::DICT(d) = deserialized.get("q").unwrap() {
        //     todo!()
        // }
        // todo!()
        // TODO: Remove
        return Err(error::SerdeError::UnimplementedQueryParsing);
    }

    // CONTINUE:

    // TODO:
    todo!()
}

pub fn serialize(ds: KRPC) -> Box<[u8]> {
    let raw = vec![0u8; 10]; // TODO: change '10' to something else

    // todo!();
    raw.into_boxed_slice()
}
