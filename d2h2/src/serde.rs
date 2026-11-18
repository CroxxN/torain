#![allow(dead_code, unused_variables)]

use std::collections::BTreeMap;

use crate::error::{self, DHTError, SerdeError};
use bencode::{bencode, BTypes};
use uttd::url::{Scheme, Url};

// quick helper function to debug u8 values in a Box<>
trait Boxy {
    fn print_box(&self) -> ();
}

impl Boxy for Box<[u8]> {
    fn print_box(&self) -> () {
        self.iter().for_each(|x| {
            if x.is_ascii() {
                print!("{}", *x as char);
            } else {
                print!("*");
            }
        });
        println!();
    }
}

#[derive(Debug)]
pub struct KRPC {
    pub transaction_id: String,
    pub message_type: MessageType,
}

#[derive(Debug)]
pub enum MessageType {
    Query(Query),
    Response(Response),
    Error(error::DHTError),
}

// NOTE: Figure out if this is really needed. Sounds like we can use a helper like this
// to easily and generically generate MessageTypes. Might be redundant.
impl MessageType {
    fn from(mtype: MessageType) {
        todo!()
    }
}

#[derive(Debug)]
pub struct Response {
    // TODO: use &[u8] instead of String?
    id: Vec<u8>,
    // for find_node responses we don't get a token response type
    token: Option<Vec<u8>>,
    response: Option<ResponseType>,
}

// TODO: implement Response handeler. Mainly for the server side of DKPRC.
impl Response {
    fn new(id: Vec<u8>, resp_type: Option<ResponseType>) -> Self {
        if let Some(r) = resp_type {
            // TODO: do something
            match r {
                ResponseType::Node(n) => {
                    todo!()
                }
                ResponseType::Values(v) => {
                    todo!()
                }
                ResponseType::Query(_) => {
                    todo!()
                }
            }
        } else {
            return Response {
                id,
                token: None,
                response: None,
            };
        }
    }
    // TODO: implement this
    fn incr_token(&mut self) {
        // self.token += 1;
    }
}

#[derive(Debug)]
pub enum ResponseType {
    // we use box here because after the `Url` and `ResponseNode` are constructed,
    // we don't need to add to them or remove from them. We can just drop them after
    // we've consumed them.
    Node(Box<[ResponseNode]>),
    Values(Box<[Url]>),
    Query(ResponseQuery),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ResponseNode {
    id: [u8; 20],
    node: Url,
}

#[derive(Debug)]
pub struct ResponseQuery {
    target: Option<Vec<u8>>,
    info_hash: Option<Vec<u8>>,
    implied_port: Option<usize>,
    port: Option<usize>,
}

#[derive(Debug)]
pub struct Query {
    method_name: QueryType,
    // A dictionary of btypes for named argument pairs. Useful for later serialization
    // Provide a utility function to parse these arguments
    // TODO: maybe use Btrees instead
    arguments: BTypes,
}

#[derive(Debug)]
pub enum QueryType {
    Ping,
    FindNode,
    FindPeer(Vec<u8>),
    AnnouncePeer(AnnounceArgs),
}

#[derive(Debug)]
pub struct AnnounceArgs {
    implied_port: u8,
    info_hash: Vec<u8>,
    port: u32,
    token: String,
}

// INFO: de
// TODO: maybe add option disable unrelated packet processing for client mode for now?
impl KRPC {
    fn from_error(transaction_id: String, mtype: MessageType) -> Result<KRPC, SerdeError> {
        if let MessageType::Error(e) = mtype {
            return Ok(KRPC {
                transaction_id,
                message_type: MessageType::Error(e),
            });
        } else {
            return Err(SerdeError::InvalidMessageType);
        }
    }
    // NOTE: seperate serialization logic into a separate function?
    pub fn new(transaction_id: String, method: QueryType, id: &[u8; 20]) -> Self {
        let mut outer_args = BTreeMap::new();
        let mut inner_args = BTreeMap::new();

        let q_name: Vec<u8>;

        inner_args.insert("id".into(), BTypes::BSTRING(id.into()));
        outer_args.insert("y".into(), BTypes::BSTRING(b"q".into()));

        match method {
            QueryType::Ping => q_name = "ping".into(),
            QueryType::FindNode => {
                q_name = "find_node".into();
                inner_args.insert("target".into(), BTypes::BSTRING(vec![0; 20]));
            }
            QueryType::FindPeer(ref p) => {
                q_name = "get_peers".into();
                inner_args.insert("info_hash".into(), BTypes::BSTRING(p.to_vec()));
            }
            // remove clones
            QueryType::AnnouncePeer(ref ap) => {
                q_name = "announce_peer".into();
                inner_args.insert("implied_port".into(), BTypes::INT(ap.implied_port.into()));
                inner_args.insert("info_hash".into(), BTypes::BSTRING(ap.info_hash.clone()));
                inner_args.insert("port".into(), BTypes::INT(ap.port.into()));
                inner_args.insert("token".into(), BTypes::BSTRING(ap.token.clone().into()));
            }
        }

        outer_args.insert("q".into(), BTypes::BSTRING(q_name));
        outer_args.insert("a".into(), BTypes::DICT(inner_args));
        outer_args.insert("t".into(), BTypes::BSTRING(transaction_id.clone().into()));

        let query = Query {
            method_name: method,
            arguments: BTypes::DICT(outer_args),
        };

        Self {
            transaction_id,
            message_type: MessageType::Query(query),
        }
    }

    pub fn deserialize_bytes(packet: Vec<u8>) -> Result<KRPC, error::SerdeError> {
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
        // These three values lets the DHT node know WHICH type of packet this current one is.
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
        //     key 'r', we can be pretty certain that we didn't encounter an error from the server (as if that had been
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
                    return Ok(Self {
                        transaction_id,
                        message_type: MessageType::Error(dht_error),
                    });
                }
            }
            return Err(error::SerdeError::UnknownDHTError);
        }
        if &y == "r" {
            let id: Vec<u8>;
            let mut token = None;
            if let BTypes::DICT(d) = deserialized.get("r").unwrap() {
                // TODO: decode the response message. Also think about how we wanna give this back to the
                // overall DHT implementation---as it's used by both the client and the server.
                // We may need a few helper functions to deal with the type (4) of the available KPRC.
                if let BTypes::BSTRING(bs) = d.get("id").unwrap() {
                    id = bs.clone();
                } else {
                    return Err(error::SerdeError::NoValidIDPresent);
                }
                if let Some(BTypes::BSTRING(bs)) = d.get("token") {
                    token = Some(bs.clone());
                }

                let mut resp = Response {
                    id,
                    token,
                    response: None,
                };

                if let Some(BTypes::BSTRING(bs)) = d.get("nodes") {
                    let ips = bs
                        .chunks(26)
                        .map(|x| {
                            // todo: use this
                            let node_id: [u8; 20] = x[0..20].try_into().unwrap();
                            let ip: [u8; 4] = x[20..24].try_into().unwrap();
                            let port = u16::from_be_bytes(x[24..26].try_into().unwrap());
                            ResponseNode {
                                id: node_id,
                                node: Url::from_ip_bytes(&ip, port, Scheme::UDP),
                            }
                        })
                        .collect();
                    resp.response = Some(ResponseType::Node(ips));
                } else if let Some(BTypes::LIST(l)) = d.get("values") {
                    // create urls from the compact url info contained in bs
                    // this containes the actual peers for the find_peers query
                    let ips = l
                        .into_iter()
                        .filter_map(|b| match b {
                            BTypes::BSTRING(bs) if bs.len() >= 6 => {
                                let ip: [u8; 4] = bs[0..4].try_into().unwrap();
                                let port = u16::from_be_bytes(bs[4..6].try_into().unwrap());
                                Some(Url::from_ip_bytes(&ip, port, Scheme::UDP))
                            }
                            _ => None,
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
            let transaction_id: String = deserialized.get("t").unwrap().try_into()?;
            let method_name: String = deserialized.get("q").unwrap().try_into()?;
            let a;
            if let BTypes::DICT(key_a) = deserialized.get("a").unwrap() {
                a = key_a;
            } else {
                return Err(error::SerdeError::InvalidMessageType);
            }
            let id: Vec<u8> = a.get("id").unwrap().try_into()?;
            let mut token: Option<Vec<u8>> = None;

            let mut resp_query = ResponseQuery {
                // id
                target: None,
                info_hash: None,
                implied_port: None,
                port: None,
            };

            match method_name.as_str() {
                "ping" => (),
                "find_node" => {
                    let target: Vec<u8> = a.get("target").unwrap().try_into()?;
                    resp_query.target = Some(target);
                }
                "get_peers" => {
                    let info_hash: Vec<u8> = a.get("info_hash").unwrap().try_into()?;
                    resp_query.info_hash = Some(info_hash);
                }
                "announce_peer" => {
                    let implied_port: usize = a.get("implied_port").unwrap().try_into()?;
                    let info_hash: Vec<u8> = a.get("info_hash").unwrap().try_into()?;
                    let port: usize = a.get("port").unwrap().try_into()?;
                    token = Some(a.get("token").unwrap().try_into()?);

                    resp_query.implied_port = Some(implied_port);
                    resp_query.info_hash = Some(info_hash);
                    resp_query.port = Some(port);
                }
                // TODO: change error type
                _ => return Err(error::SerdeError::InvalidMessageType),
            }

            // Change the ergonomic of how it looks
            return Ok(KRPC {
                transaction_id,
                message_type: MessageType::Response(Response {
                    id,
                    token,
                    response: Some(ResponseType::Query(resp_query)),
                }),
            });
        }
        return Err(error::SerdeError::BencodeGenericError);
    }
    // TODO: probably doesn't need to use match, just pass the BTree to ser()
    pub fn serialize(self) -> Box<[u8]> {
        // let mut raw: Vec<u8> = Vec::new(); // TODO: change '10' to something else
        match self.message_type {
            MessageType::Query(q) => {
                return ::bencode::benencode::ser(&q.arguments).into();
            }
            MessageType::Response(_) | MessageType::Error(_) => {
                unimplemented!();
            }
        };
    }
}

// TODO: write some tests for the deserializer
//
#[cfg(test)]
mod test {
    use uttd::url::Scheme;

    // use super::serialize;
    use super::{MessageType, KRPC};
    use crate::serde::{AnnounceArgs, Response};

    #[test]
    fn deserialize_response_transaction_id() {
        let bytes = "d1:rd2:id20:mnopqrstuvwxyz123456e1:t2:aa1:y1:re"
            .as_bytes()
            .to_vec();
        let res = KRPC::deserialize_bytes(bytes).unwrap();
        assert_eq!(res.transaction_id, "aa")
    }
    #[test]
    fn deserialize_response_transaction_id_failed() {
        let bytes = "d1:rd2:id20:mnopqrstuvwxyz123456e1:t2:aa1:y1:re"
            .as_bytes()
            .to_vec();
        let res = KRPC::deserialize_bytes(bytes).unwrap();
        assert!(res.transaction_id != "bb")
    }
    #[test]
    fn deserialize_response_id() {
        let bytes = "d1:rd2:id20:mnopqrstuvwxyz123456e1:t2:aa1:y1:re"
            .as_bytes()
            .to_vec();
        let res = KRPC::deserialize_bytes(bytes).unwrap();
        let expected_res = Response {
            id: b"mnopqrstuvwxyz123456".to_vec(),
            token: None,
            response: None,
        };
        if let MessageType::Response(r) = res.message_type {
            assert_eq!(r.id, expected_res.id);
            assert_eq!(r.token, expected_res.token);
            // assert!(r.response.as_ref() == None);
        } else {
            panic!("Error")
        }
    }
    #[test]
    fn deserialize_peers() {
        let bytes =
            b"d1:rd2:id20:abcdefghij01234567895:token8:deadbeef6:valuesl6:\x7f\x00\x00\x01\x1a\xe1ee1:t2:aa1:y1:re"
                .to_vec();
        let res = KRPC::deserialize_bytes(bytes).unwrap();
        if let MessageType::Response(r) = res.message_type {
            assert_eq!(r.id, b"abcdefghij0123456789".to_vec());
            if let Some(super::ResponseType::Values(nodes)) = r.response {
                assert_eq!(nodes[0].host, "127.0.0.1:6881");
                assert_eq!(nodes[0].scheme, Scheme::UDP);
                assert_eq!(nodes[0].port(), 6881);
            } else {
                panic!("Message Response Type is not 'find_node': {:?}", r)
            }
        } else {
            panic!("Message Type is not response")
        }
    }
    #[test]
    fn deserialize_nodes() {
        let bytes =
            b"d1:rd2:id20:abcdefghij01234567895:nodes26:00000000000000000001\x7f\x00\x00\x01\x1a\xe1e1:t2:aa1:y1:re"
                .to_vec();
        let res = KRPC::deserialize_bytes(bytes).unwrap();
        if let MessageType::Response(r) = res.message_type {
            if let Some(super::ResponseType::Node(nodes)) = r.response {
                assert_eq!(&nodes[0].id, b"00000000000000000001");
                assert_eq!(nodes[0].node.host, "127.0.0.1:6881");
                assert_eq!(nodes[0].node.scheme, Scheme::UDP);
                assert_eq!(nodes[0].node.port(), 6881);
            } else {
                panic!("Message Response Type is not 'find_node': {:?}", r)
            }
        } else {
            panic!("Message Type is not response")
        }
    }

    #[test]
    fn test_serialize() {
        let krpc = KRPC::new("aa".into(), super::QueryType::Ping, &[0; 20]);
        let serialized_krpc = krpc.serialize();
        // currently fails because deserialization for Query isn't implemented, only Response.
        let deserialized_krpc = KRPC::deserialize_bytes(serialized_krpc.to_vec()).unwrap();
        assert_eq!(deserialized_krpc.transaction_id, "aa");
    }
    #[test]
    fn serializae_announce() {
        let ap = AnnounceArgs {
            implied_port: 1,
            info_hash: vec![0; 20],
            port: 6881,
            token: "aoeusnth".into(),
        };

        let krpc = KRPC::new("aa".into(), super::QueryType::AnnouncePeer(ap), &[0; 20]);
        let serialized_krpc = krpc.serialize();
        // TODO: assert a more meaningful metric
        assert!(serialized_krpc.len() != 0);
    }
}
