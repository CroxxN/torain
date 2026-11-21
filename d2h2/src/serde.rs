use std::collections::BTreeMap;

use bencode::bencode::BTypes;
use uttd::url::{Scheme, Url};

use crate::error::{self, DHTError, SerdeError};

#[derive(Debug, PartialEq, Clone)]
pub struct KRPC {
    pub transact_id: [u8; 2],
    pub message_type: MessageType,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MessageType {
    Query {
        id: [u8; 20],
        query: QueryType,
    },
    Response {
        id: [u8; 20],
        response: ResponseType,
    },
    Error(error::DHTError),
}

// Empty tags. Makes life easier to decode responses
#[derive(Debug, PartialEq, Clone)]
pub enum MessageTag {
    Ping,
    FindNode,
    GetPeers,
    AnnouncePeer,
}

#[derive(Debug, PartialEq, Clone)]
pub enum QueryType {
    Ping,
    FindNode {
        target: [u8; 20],
    },
    GetPeers {
        info_hash: [u8; 20],
    },
    AnnouncePeer {
        implied_port: bool,
        info_hash: [u8; 20],
        port: u32,
        token: [u8; 20],
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum ResponseType {
    Ping,
    FindNode {
        nodes: Vec<([u8; 20], Url)>,
    },
    GetPeers {
        token: Option<[u8; 20]>, // though the BEP 005 says token is required, that's not the case practically. Many nodes don't treat the token like optional
        vals: Peers,
    },
    AnnouncePeer,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Peers {
    Values(Vec<Url>),
    Nodes(Vec<([u8; 20], Url)>),
}

pub trait Vec2Kprc {
    type Error;
    fn into(message: Vec<u8>, hint: Option<MessageTag>) -> Result<Self, Self::Error>
    where
        Self: Sized;
    fn de_error(error: &BTypes) -> Result<MessageType, Self::Error>;
    fn de_response(response: &BTypes, hint: MessageTag) -> Result<MessageType, Self::Error>;
    fn de_query(query: &BTypes, query_string: String) -> Result<MessageType, Self::Error>;
}

impl KRPC {
    pub fn new(transaction_id: [u8; 2], method: MessageType) -> Self {
        Self {
            transact_id: transaction_id,
            message_type: method,
        }
    }
    pub fn serialize(self) -> Vec<u8> {
        let btrees = self.btrees();
        bencode::benencode::ser(&btrees)
    }

    fn btrees(self) -> BTypes {
        match self.message_type {
            MessageType::Query { id, query } => Self::query_btrees(self.transact_id, id, query),
            MessageType::Response { id, response } => {
                Self::response_btrees(self.transact_id, id, response)
            }
            MessageType::Error(error) => Self::error_btrees(self.transact_id, error),
        }
    }

    fn query_btrees(tid: [u8; 2], id: [u8; 20], query: QueryType) -> BTypes {
        let mut outer_args: BTreeMap<String, BTypes> = BTreeMap::new();
        let mut inner_args = BTreeMap::new();

        let q_name: Vec<u8>;

        inner_args.insert("id".into(), BTypes::BSTRING(id.into()));
        outer_args.insert("y".into(), BTypes::BSTRING(b"q".into()));

        match query {
            QueryType::Ping => q_name = "ping".into(),
            QueryType::FindNode { target } => {
                q_name = "find_node".into();
                inner_args.insert("target".into(), BTypes::BSTRING(target.to_vec()));
            }
            QueryType::GetPeers { info_hash } => {
                q_name = "get_peers".into();
                inner_args.insert("info_hash".into(), BTypes::BSTRING(info_hash.to_vec()));
            }
            // remove clones
            QueryType::AnnouncePeer {
                implied_port,
                info_hash,
                port,
                token,
            } => {
                q_name = "announce_peer".into();
                inner_args.insert("implied_port".into(), BTypes::INT(implied_port.into()));
                inner_args.insert("info_hash".into(), BTypes::BSTRING(info_hash.to_vec()));
                inner_args.insert("port".into(), BTypes::INT(port.into()));
                inner_args.insert("token".into(), BTypes::BSTRING(token.to_vec()));
            }
        }

        outer_args.insert("q".into(), BTypes::BSTRING(q_name));
        outer_args.insert("a".into(), BTypes::DICT(inner_args));
        outer_args.insert("t".into(), BTypes::BSTRING(tid.into()));

        BTypes::DICT(outer_args)
    }
    fn response_btrees(tid: [u8; 2], id: [u8; 20], response: ResponseType) -> BTypes {
        let mut outer_args: BTreeMap<String, BTypes> = BTreeMap::new();
        let mut inner_args = BTreeMap::new();

        inner_args.insert("id".into(), BTypes::BSTRING(id.into()));
        outer_args.insert("y".into(), BTypes::BSTRING(b"r".into()));

        match response {
            ResponseType::Ping | ResponseType::AnnouncePeer => (),
            ResponseType::FindNode { nodes } => {
                let mut ip_node = Vec::new();
                nodes.into_iter().for_each(|(id, url)| {
                    ip_node.extend_from_slice(&id);
                    ip_node.extend_from_slice(url.to_bytes().unwrap().as_slice());
                });
                inner_args.insert("nodes".into(), BTypes::BSTRING(ip_node));
            }
            ResponseType::GetPeers { token, vals } => {
                let mut ip_node = Vec::new();
                if let Peers::Nodes(nodes) = vals {
                    nodes.into_iter().for_each(|(id, url)| {
                        ip_node.extend_from_slice(&id);
                        ip_node.extend_from_slice(url.to_bytes().unwrap().as_slice());
                    });
                    inner_args.insert("nodes".into(), BTypes::BSTRING(ip_node));
                } else if let Peers::Values(v) = vals {
                    v.into_iter().for_each(|url| {
                        ip_node.extend_from_slice(url.to_bytes().unwrap().as_slice());
                    });
                    inner_args.insert("values".into(), BTypes::BSTRING(ip_node));
                }
                if let Some(t) = token {
                    inner_args.insert("token".into(), BTypes::BSTRING(t.into()));
                }
            }
        }

        outer_args.insert("r".into(), BTypes::DICT(inner_args));
        outer_args.insert("t".into(), BTypes::BSTRING(tid.into()));
        BTypes::DICT(outer_args)
    }
    fn error_btrees(tid: [u8; 2], e: DHTError) -> BTypes {
        let mut outer_args: BTreeMap<String, BTypes> = BTreeMap::new();

        outer_args.insert("y".into(), BTypes::BSTRING(b"e".into()));
        outer_args.insert("t".into(), BTypes::BSTRING(tid.into()));

        let code = BTypes::INT(e.error_code().into());
        let message = BTypes::BSTRING(e.error_message().into());

        let combined_message = vec![code, message];
        outer_args.insert("e".into(), BTypes::LIST(combined_message));
        BTypes::DICT(outer_args)
    }
}

impl Vec2Kprc for KRPC {
    type Error = error::SerdeError;
    fn into(message: Vec<u8>, hint: Option<MessageTag>) -> Result<Self, Self::Error> {
        let deserialized =
            if let BTypes::DICT(d) = bencode::bencode::decode(&mut message.into_iter())? {
                d
            } else {
                // TODO: fix this
                return Err(error::SerdeError::BencodeGenericError);
            };

        let transaction_id: Vec<u8> = deserialized.get("t").unwrap().try_into()?;
        let y: String = deserialized.get("y").unwrap().try_into()?;
        // The key 'y' can have one of three values:
        //     a. 'e': Error
        //     b. 'r': Response Packet
        //     c. 'q': Query Packet -> 'q' is named, 'a' contains the actual arguments
        let message = match y.as_str() {
            "e" => Self::de_error(deserialized.get("e").unwrap()),
            "r" => Self::de_response(deserialized.get("r").unwrap(), hint.unwrap()),
            "q" => Self::de_query(
                deserialized.get("a").unwrap(),
                deserialized.get("q").unwrap().try_into().unwrap(),
            ),
            _ => {
                return Err(SerdeError::InvalidMessageType);
            }
        };
        let message = message.unwrap();
        Ok(Self {
            transact_id: transaction_id.try_into().unwrap(), // remove unwrap
            message_type: message,
        })
    }

    fn de_error(error: &BTypes) -> Result<MessageType, SerdeError> {
        let dht_error;
        if let &BTypes::LIST(l) = &error {
            if let BTypes::INT(bint) = l[0] {
                match bint {
                    201 => dht_error = DHTError::GenericError,
                    202 => dht_error = DHTError::ServerError,
                    203 => dht_error = DHTError::ProtocError,
                    _ => dht_error = DHTError::UnknownMethod,
                }
                return Ok(MessageType::Error(dht_error));
            };
        };
        Err(error::SerdeError::UnknownDHTError)
    }

    fn de_response(response: &BTypes, hint: MessageTag) -> Result<MessageType, SerdeError> {
        // Variant 2
        let d = if let BTypes::DICT(d) = response {
            d
        } else {
            return Err(error::SerdeError::UnknownDHTError);
        };

        let id = if let BTypes::BSTRING(bs) = d.get("id").unwrap() {
            bs.clone().try_into().unwrap()
        } else {
            return Err(error::SerdeError::NoValidIDPresent);
        };

        match hint {
            MessageTag::Ping | MessageTag::AnnouncePeer => {
                let response = if hint == MessageTag::Ping {
                    ResponseType::Ping
                } else {
                    ResponseType::AnnouncePeer
                };
                Ok(MessageType::Response { id: id, response })
            }
            // TODO: combine FindNode and GetPeers
            MessageTag::FindNode | MessageTag::GetPeers => {
                let token: Option<[u8; 20]> = if let Some(BTypes::BSTRING(bs)) = d.get("token") {
                    let mut tks = [0u8; 20];
                    tks[0..8].copy_from_slice(bs.as_slice());
                    Some(tks)
                } else {
                    None
                };
                if let Some(BTypes::BSTRING(bs)) = d.get("nodes") {
                    let ips = bs
                        .chunks(26)
                        .map(|x| {
                            let node_id: [u8; 20] = x[0..20].try_into().unwrap();
                            let ip: [u8; 4] = x[20..24].try_into().unwrap();
                            let port = u16::from_be_bytes(x[24..26].try_into().unwrap());
                            (node_id, Url::from_ip_bytes(&ip, port, Scheme::UDP))
                        })
                        .collect();
                    let response = if hint == MessageTag::FindNode {
                        ResponseType::FindNode { nodes: ips }
                    } else {
                        ResponseType::GetPeers {
                            token,
                            vals: Peers::Nodes(ips),
                        }
                    };
                    Ok(MessageType::Response { id, response })
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
                    Ok(MessageType::Response {
                        id,
                        response: ResponseType::GetPeers {
                            token,
                            vals: Peers::Values(ips),
                        },
                    })
                } else {
                    Err(error::SerdeError::UnimplementedQueryParsing)
                }
            }
        }
    }
    fn de_query(query: &BTypes, query_string: String) -> Result<MessageType, Self::Error> {
        let a;
        if let BTypes::DICT(key_a) = query {
            a = key_a;
        } else {
            return Err(error::SerdeError::InvalidMessageType);
        }

        let id: Vec<u8> = a.get("id").unwrap().try_into()?;
        let id = id.try_into().unwrap();

        match query_string.as_str() {
            "ping" => Ok(MessageType::Query {
                id,
                query: QueryType::Ping,
            }),
            "find_node" => {
                let target = a.get("target").unwrap().try_into()?;
                Ok(MessageType::Query {
                    id,
                    query: QueryType::FindNode { target },
                })
            }
            "get_peers" => {
                let info_hash = a.get("info_hash").unwrap().try_into()?;
                Ok(MessageType::Query {
                    id,
                    query: QueryType::GetPeers { info_hash },
                })
            }
            "announce_peer" => {
                let implied_port: usize = a.get("implied_port").unwrap().try_into()?;
                let implied_port = implied_port != 0;

                let info_hash = a.get("info_hash").unwrap().try_into()?;

                let port: usize = a.get("port").unwrap().try_into()?;
                let port = port as u32;

                let token = a.get("token").unwrap().try_into()?;

                Ok(MessageType::Query {
                    id,
                    query: QueryType::AnnouncePeer {
                        implied_port,
                        info_hash,
                        port,
                        token,
                    },
                })
            }
            // TODO: change error type
            _ => return Err(error::SerdeError::InvalidMessageType),
        }
    }
}

//
#[cfg(test)]
mod test {

    use uttd::url::Url;

    use crate::serde::{
        MessageTag::{self, Ping},
        Peers, QueryType, ResponseType, Vec2Kprc,
    };

    use super::{MessageType, KRPC};

    #[test]
    fn deserialize_response_transaction_id() {
        let bytes = "d1:rd2:id20:mnopqrstuvwxyz123456e1:t2:aa1:y1:re"
            .as_bytes()
            .to_vec();
        let res: KRPC = Vec2Kprc::into(bytes, Some(MessageTag::Ping)).unwrap();
        assert_eq!(res.transact_id, ['a' as u8, 'a' as u8])
    }
    #[test]
    fn deserialize_response_transaction_id_failed() {
        let bytes = "d1:rd2:id20:mnopqrstuvwxyz123456e1:t2:aa1:y1:re"
            .as_bytes()
            .to_vec();
        let res: KRPC = Vec2Kprc::into(bytes, Some(MessageTag::Ping)).unwrap();
        assert_ne!(res.transact_id, ['b' as u8, 'b' as u8])
    }
    #[test]
    fn deserialize_response_id() {
        let bytes = "d1:rd2:id20:mnopqrstuvwxyz123456e1:t2:aa1:y1:re"
            .as_bytes()
            .to_vec();

        let res: KRPC = Vec2Kprc::into(bytes, Some(MessageTag::Ping)).unwrap();

        let expected_res = KRPC {
            transact_id: ['a' as u8, 'a' as u8],
            message_type: MessageType::Response {
                id: b"mnopqrstuvwxyz123456".to_vec().try_into().unwrap(),
                response: ResponseType::Ping,
            },
        };

        assert_eq!(res, expected_res);
    }
    #[test]
    fn deserialize_peers() {
        let bytes =
            b"d1:rd2:id20:abcdefghij01234567895:token8:deadbeef6:valuesl6:\x7f\x00\x00\x01\x1a\xe1ee1:t2:aa1:y1:re"
                .to_vec();
        let res: KRPC = Vec2Kprc::into(bytes, Some(MessageTag::GetPeers)).unwrap();

        let mut token = [0u8; 20];
        token[0..8].copy_from_slice(b"deadbeef");
        let token = Some(token);
        let urls = vec![Url::new("udp://127.0.0.1:6881").unwrap()];

        let expected_response = ResponseType::GetPeers {
            token,
            vals: Peers::Values(urls),
        };

        assert_eq!(
            res,
            KRPC {
                transact_id: [b'a', b'a'],
                message_type: MessageType::Response {
                    id: b"abcdefghij0123456789".to_owned().try_into().unwrap(),
                    response: expected_response
                }
            }
        );
    }
    #[test]
    fn deserialize_nodes() {
        let bytes =
            b"d1:rd2:id20:abcdefghij01234567895:nodes26:00000000000000000001\x7f\x00\x00\x01\x1a\xe1e1:t2:aa1:y1:re"
                .to_vec();
        let res: KRPC = Vec2Kprc::into(bytes, Some(MessageTag::GetPeers)).unwrap();

        let url = Url::new("udp://127.0.0.1:6881").unwrap();

        let expected_response = ResponseType::GetPeers {
            token: None,
            vals: Peers::Nodes(vec![(b"00000000000000000001".to_owned().into(), url)]),
        };
        assert_eq!(
            res,
            KRPC {
                transact_id: [b'a', b'a'],
                message_type: MessageType::Response {
                    id: b"abcdefghij0123456789".to_owned().try_into().unwrap(),
                    response: expected_response
                }
            }
        );
    }

    #[test]
    fn test_serialize() {
        let message = MessageType::Query {
            id: [0; 20],
            query: crate::serde::QueryType::Ping,
        };
        let krpc = KRPC::new(['a' as u8, 'a' as u8], message);
        let serialized_krpc = krpc.clone().serialize();
        let deserialized_krpc: KRPC = Vec2Kprc::into(serialized_krpc, None).unwrap();
        assert_eq!(krpc, deserialized_krpc);
    }

    #[test]
    fn test_serialized_response() {
        let message = MessageType::Response {
            id: [0; 20],
            response: crate::serde::ResponseType::Ping,
        };
        let krpc = KRPC::new(['a' as u8, 'a' as u8], message);
        let serialized_krpc = krpc.clone().serialize();
        let deserialized_krpc: KRPC = Vec2Kprc::into(serialized_krpc, Some(Ping)).unwrap();
        assert_eq!(krpc, deserialized_krpc);
    }

    #[test]
    fn serializae_announce() {
        let ap = QueryType::AnnouncePeer {
            implied_port: true,
            info_hash: [0; 20],
            port: 6881,
            token: b"aoeusnthaoeusnthaaaa".to_owned(),
        };

        let message = MessageType::Query {
            id: [0; 20],
            query: ap,
        };
        let krpc = KRPC::new(['a' as u8, 'a' as u8], message);

        let serialized_krpc = krpc.clone().serialize();

        let deserialized_krpc: KRPC = Vec2Kprc::into(serialized_krpc, None).unwrap();
        assert_eq!(krpc, deserialized_krpc);
    }
}
