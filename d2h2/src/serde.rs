#![allow(dead_code, unused_variables)]

use std::collections::BTreeMap;

use crate::error::{self, DHTError, SerdeError};
use ::bencode::benencode;
use ::bencode::utils::vec_to_string;
use bencode::{bencode, BTypes};
use uttd::url::{Scheme, Url};

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

#[derive(Debug, PartialEq, Eq)]
pub struct Response {
    // TODO: use &[u8] instead of String?
    pub id: String,
    // for find_node responses we don't get a token response type
    pub token: Option<String>,
    pub response: Option<ResponseType>,
}

impl Response {
    pub fn new(id: String, token: Option<String>, resp_type: Option<ResponseType>) -> Self {
        Response {
            id,
            token,
            response: resp_type,
        }
    }

    /// Serialize this response body into a BTypes::DICT suitable for the "r" key
    fn to_btypes(&self) -> BTypes {
        let mut r = BTreeMap::new();
        r.insert("id".into(), BTypes::BSTRING(self.id.bytes().collect()));

        if let Some(ref token) = self.token {
            r.insert("token".into(), BTypes::BSTRING(token.bytes().collect()));
        }

        match &self.response {
            Some(ResponseType::Node(nodes)) => {
                // compact node info: 20-byte id + 4-byte IP + 2-byte port per node
                let mut compact = Vec::with_capacity(nodes.len() * 26);
                for node in nodes.iter() {
                    compact.extend_from_slice(&node.id);
                    compact.extend_from_slice(&compact_peer_bytes(&node.node));
                }
                r.insert("nodes".into(), BTypes::BSTRING(compact));
            }
            Some(ResponseType::Values(peers)) => {
                // values: list of compact 6-byte peer info strings
                let values: Vec<BTypes> = peers
                    .iter()
                    .map(|url| BTypes::BSTRING(compact_peer_bytes(url).to_vec()))
                    .collect();
                r.insert("values".into(), BTypes::LIST(values));
            }
            None => {}
        }

        BTypes::DICT(r)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResponseType {
    // we use box here because after the `Url` and `ResponseNode` are constructed,
    // we don't need to add to them or remove from them. We can just drop them after
    // we've consumed them.
    Node(Box<[ResponseNode]>),
    Values(Box<[Url]>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ResponseNode {
    pub id: [u8; 20],
    pub node: Url,
}

#[derive(Debug)]
pub struct Query {
    pub method_name: QueryType,
    // A dictionary of btypes for named argument pairs. Useful for later serialization
    pub arguments: QueryDataTypes,
}

impl Query {
    /// Extract a byte-slice argument from the "a" dict by key name.
    /// Returns None if the key doesn't exist or isn't a QVec.
    pub fn get_arg(&self, key: &str) -> Option<&[u8]> {
        if let QueryDataTypes::QDict(ref outer) = self.arguments {
            // arguments may be flat (from deserialized queries) or nested under "a" (from new())
            // try flat first
            if let Some(QueryDataTypes::QVec(v)) = outer.get(key) {
                return Some(v);
            }
            // try nested under "a"
            if let Some(QueryDataTypes::QDict(a)) = outer.get("a") {
                if let Some(QueryDataTypes::QVec(v)) = a.get(key) {
                    return Some(v);
                }
            }
        }
        None
    }
}

#[derive(Debug)]
pub enum QueryType {
    Ping,
    FindNode,
    FindPeer,
    AnnouncePeer,
}

#[derive(Debug)]
pub enum QueryDataTypes {
    QVec(Box<[u8]>),
    QDict(BTreeMap<String, QueryDataTypes>),
}

impl From<QueryDataTypes> for BTypes {
    fn from(value: QueryDataTypes) -> Self {
        match value {
            QueryDataTypes::QVec(v) => BTypes::BSTRING(v.into()),
            QueryDataTypes::QDict(d) => {
                let mut map = BTreeMap::new();
                for (k, v) in d {
                    map.insert(k, BTypes::from(v));
                }
                BTypes::DICT(map)
            }
        }
    }
}

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
    pub fn new(transaction_id: String, method: QueryType, id: &[u8; 20]) -> Self {
        let mut arguments_a = BTreeMap::new();
        let mut a_argument_b = BTreeMap::new();

        let q_name: Vec<u8> = match method {
            QueryType::Ping => "ping".into(),
            QueryType::FindNode => "find_node".into(),
            QueryType::FindPeer => "get_peers".into(),
            QueryType::AnnouncePeer => "announce_peer".into(),
        };

        a_argument_b.insert("id".into(), QueryDataTypes::QVec(id.to_vec().into()));
        arguments_a.insert("a".into(), QueryDataTypes::QDict(a_argument_b));
        arguments_a.insert("y".into(), QueryDataTypes::QVec("q".as_bytes().into()));
        arguments_a.insert("q".into(), QueryDataTypes::QVec(q_name.into()));
        // TODO: add key "t" here?
        arguments_a.insert(
            "t".into(),
            QueryDataTypes::QVec(transaction_id.as_bytes().into()),
        );
        // arguments.
        let query = Query {
            method_name: method,
            arguments: QueryDataTypes::QDict(arguments_a),
        };
        Self {
            transaction_id,
            message_type: MessageType::Query(query),
        }
    }

    // TODO: think of a better way to handle arguments
    pub fn set_token(&mut self, token: &str) {
        self.set_token_bytes(token.as_bytes());
    }

    pub fn set_token_bytes(&mut self, token: &[u8]) {
        if let MessageType::Query(q) = &mut self.message_type {
            match &mut q.arguments {
                QueryDataTypes::QDict(qd) => {
                    if let Some(a) = qd.get_mut("a") {
                        if let QueryDataTypes::QDict(b) = a {
                            b.insert("token".into(), QueryDataTypes::QVec(token.to_vec().into()));
                        }
                    }
                }
                _ => unimplemented!(),
            }
        }
    }

    pub fn set_info_hash(&mut self, info_hash: &[u8; 20]) {
        if let MessageType::Query(q) = &mut self.message_type {
            if let QueryDataTypes::QDict(qd) = &mut q.arguments {
                if let Some(QueryDataTypes::QDict(a)) = qd.get_mut("a") {
                    a.insert(
                        "info_hash".into(),
                        QueryDataTypes::QVec(info_hash.to_vec().into()),
                    );
                }
            }
        }
    }

    pub fn set_target(&mut self, target: &[u8; 20]) {
        if let MessageType::Query(q) = &mut self.message_type {
            if let QueryDataTypes::QDict(qd) = &mut q.arguments {
                if let Some(QueryDataTypes::QDict(a)) = qd.get_mut("a") {
                    a.insert(
                        "target".into(),
                        QueryDataTypes::QVec(target.to_vec().into()),
                    );
                }
            }
        }
    }

    pub fn set_port(&mut self, port: u16) {
        if let MessageType::Query(q) = &mut self.message_type {
            if let QueryDataTypes::QDict(qd) = &mut q.arguments {
                if let Some(QueryDataTypes::QDict(a)) = qd.get_mut("a") {
                    a.insert(
                        "port".into(),
                        QueryDataTypes::QVec(port.to_be_bytes().to_vec().into()),
                    );
                }
            }
        }
    }

    pub fn set_implied_port(&mut self, implied: bool) {
        if let MessageType::Query(q) = &mut self.message_type {
            if let QueryDataTypes::QDict(qd) = &mut q.arguments {
                if let Some(QueryDataTypes::QDict(a)) = qd.get_mut("a") {
                    let val = if implied { 1u8 } else { 0u8 };
                    a.insert(
                        "implied_port".into(),
                        QueryDataTypes::QVec(vec![val].into()),
                    );
                }
            }
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
                    return Ok(Self {
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
                if let Some(BTypes::BSTRING(bs)) = d.get("token") {
                    token = Some(vec_to_string(bs));
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

        // Incoming query from another DHT node
        if &y == "q" {
            let method_name: String = deserialized
                .get("q")
                .ok_or(SerdeError::BencodeGenericError)?
                .try_into()?;
            let method = match method_name.as_str() {
                "ping" => QueryType::Ping,
                "find_node" => QueryType::FindNode,
                "get_peers" => QueryType::FindPeer,
                "announce_peer" => QueryType::AnnouncePeer,
                _ => return Err(SerdeError::InvalidMessageType),
            };

            // parse the "a" arguments dict into QueryDataTypes
            let arguments = if let Some(BTypes::DICT(a_dict)) = deserialized.get("a") {
                let mut qd = BTreeMap::new();
                for (k, v) in a_dict {
                    let qval = match v {
                        BTypes::BSTRING(bs) => QueryDataTypes::QVec(bs.clone().into()),
                        BTypes::DICT(inner) => {
                            let mut inner_qd = BTreeMap::new();
                            for (ik, iv) in inner {
                                if let BTypes::BSTRING(ibs) = iv {
                                    inner_qd.insert(
                                        ik.clone(),
                                        QueryDataTypes::QVec(ibs.clone().into()),
                                    );
                                }
                            }
                            QueryDataTypes::QDict(inner_qd)
                        }
                        _ => continue,
                    };
                    qd.insert(k.clone(), qval);
                }
                QueryDataTypes::QDict(qd)
            } else {
                QueryDataTypes::QDict(BTreeMap::new())
            };

            let query = Query {
                method_name: method,
                arguments,
            };
            return Ok(KRPC {
                transaction_id,
                message_type: MessageType::Query(query),
            });
        }

        Err(error::SerdeError::InvalidMessageType)
    }
}

/// Convert a Url's host:port into compact 6-byte peer info (4 IP + 2 port)
fn compact_peer_bytes(url: &Url) -> [u8; 6] {
    let mut out = [0u8; 6];
    let (ip_str, port_str) = url.host.rsplit_once(':').unwrap_or((&url.host, "0"));
    let port: u16 = port_str.parse().unwrap_or(0);
    let parts: Vec<&str> = ip_str.split('.').collect();
    if parts.len() == 4 {
        for (i, p) in parts.iter().enumerate() {
            out[i] = p.parse().unwrap_or(0);
        }
    }
    let port_bytes = port.to_be_bytes();
    out[4] = port_bytes[0];
    out[5] = port_bytes[1];
    out
}

// TODO: probably make this a macro_rules!
pub fn serialize(ds: KRPC) -> Box<[u8]> {
    let mut outer = BTreeMap::new();
    outer.insert("t".into(), BTypes::BSTRING(ds.transaction_id.into_bytes()));

    match ds.message_type {
        MessageType::Query(q) => {
            // Query: reuse existing BTypes conversion for the full dict
            let btypes = BTypes::from(q.arguments);
            return benencode::ser(&btypes).into_boxed_slice();
        }
        MessageType::Response(resp) => {
            outer.insert("y".into(), BTypes::BSTRING(b"r".to_vec()));
            outer.insert("r".into(), resp.to_btypes());
        }
        MessageType::Error(ref e) => {
            outer.insert("y".into(), BTypes::BSTRING(b"e".to_vec()));
            let (code, msg) = match e {
                DHTError::GenericError => (201i64, "A Generic Error Occurred"),
                DHTError::ServerError => (202, "A Server Error Occurred"),
                DHTError::ProtocError => {
                    (203, "Protocol Error: Malformed Packet or Invalid Arguments")
                }
                DHTError::UnknownMethod => (204, "Method Unknown"),
                DHTError::FailedSystemTimeGen(_) => (202, "Internal Server Error"),
            };
            outer.insert(
                "e".into(),
                BTypes::LIST(vec![
                    BTypes::INT(code),
                    BTypes::BSTRING(msg.as_bytes().to_vec()),
                ]),
            );
        }
    }

    benencode::ser(&BTypes::DICT(outer)).into_boxed_slice()
}

// TODO: write some tests for the deserializer
//
#[cfg(test)]
mod test {
    use uttd::url::Scheme;

    use super::{MessageType, KRPC};
    use crate::serde::Response;

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
            id: "mnopqrstuvwxyz123456".to_string(),
            token: None,
            response: None,
        };
        if let MessageType::Response(r) = res.message_type {
            assert_eq!(r.id, expected_res.id);
            assert_eq!(r.token, expected_res.token);
            assert!(r.response.as_ref() == None);
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
            assert_eq!(r.id, "abcdefghij0123456789".to_string());
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
    fn serialize_ping() {
        let krpc = KRPC::new("aa".into(), super::QueryType::Ping, &[0; 20]);
        let serialized = super::serialize(krpc);
        // verify it's valid bencode by deserializing back
        let decoded = bencode::bencode::decode(&mut serialized.to_vec().into_iter()).unwrap();
        if let bencode::bencode::BTypes::DICT(d) = decoded {
            // should have keys: a, q, t, y
            assert!(d.contains_key("a"));
            assert!(d.contains_key("q"));
            assert!(d.contains_key("t"));
            assert!(d.contains_key("y"));
        } else {
            panic!("Serialized KRPC is not a dict")
        }
    }

    #[test]
    fn serialize_roundtrip_find_node() {
        let node_id = b"abcdefghij0123456789";
        let krpc = KRPC::new("aa".into(), super::QueryType::FindNode, node_id);
        let serialized = super::serialize(krpc);
        // the serialized bytes should be valid bencode
        let decoded = bencode::bencode::decode(&mut serialized.to_vec().into_iter()).unwrap();
        if let bencode::bencode::BTypes::DICT(d) = decoded {
            let q: String = d.get("q").unwrap().try_into().unwrap();
            assert_eq!(q, "find_node");
            let y: String = d.get("y").unwrap().try_into().unwrap();
            assert_eq!(y, "q");
            let t: String = d.get("t").unwrap().try_into().unwrap();
            assert_eq!(t, "aa");
        } else {
            panic!("Serialized KRPC is not a dict")
        }
    }

    #[test]
    fn serialize_response_ping() {
        let resp = super::Response::new("abcdefghij0123456789".into(), None, None);
        let krpc = KRPC {
            transaction_id: "pg".into(),
            message_type: MessageType::Response(resp),
        };
        let serialized = super::serialize(krpc);
        let decoded = bencode::bencode::decode(&mut serialized.to_vec().into_iter()).unwrap();
        if let bencode::bencode::BTypes::DICT(d) = decoded {
            let y: String = d.get("y").unwrap().try_into().unwrap();
            assert_eq!(y, "r");
            let t: String = d.get("t").unwrap().try_into().unwrap();
            assert_eq!(t, "pg");
            if let bencode::bencode::BTypes::DICT(r) = d.get("r").unwrap() {
                let id: String = r.get("id").unwrap().try_into().unwrap();
                assert_eq!(id, "abcdefghij0123456789");
            } else {
                panic!("Expected dict for 'r' key")
            }
        } else {
            panic!("Serialized KRPC is not a dict")
        }
    }

    #[test]
    fn serialize_response_with_nodes() {
        let nodes = vec![super::ResponseNode {
            id: *b"00000000000000000001",
            node: uttd::url::Url::from_ip_bytes(&[127, 0, 0, 1], 6881, Scheme::UDP),
        }];
        let resp = super::Response::new(
            "abcdefghij0123456789".into(),
            None,
            Some(super::ResponseType::Node(nodes.into())),
        );
        let krpc = KRPC {
            transaction_id: "fn".into(),
            message_type: MessageType::Response(resp),
        };
        let serialized = super::serialize(krpc);
        // roundtrip: deserialize and verify
        let parsed = KRPC::deserialize_bytes(serialized.to_vec()).unwrap();
        assert_eq!(parsed.transaction_id, "fn");
        if let MessageType::Response(r) = parsed.message_type {
            if let Some(super::ResponseType::Node(nodes)) = r.response {
                assert_eq!(nodes.len(), 1);
                assert_eq!(&nodes[0].id, b"00000000000000000001");
                assert_eq!(nodes[0].node.port(), 6881);
            } else {
                panic!("Expected Node response type")
            }
        } else {
            panic!("Expected Response")
        }
    }

    #[test]
    fn serialize_response_with_values() {
        let peers = vec![uttd::url::Url::from_ip_bytes(
            &[127, 0, 0, 1],
            6881,
            Scheme::UDP,
        )];
        let resp = super::Response::new(
            "abcdefghij0123456789".into(),
            Some("deadbeef".into()),
            Some(super::ResponseType::Values(peers.into())),
        );
        let krpc = KRPC {
            transaction_id: "gp".into(),
            message_type: MessageType::Response(resp),
        };
        let serialized = super::serialize(krpc);
        let parsed = KRPC::deserialize_bytes(serialized.to_vec()).unwrap();
        assert_eq!(parsed.transaction_id, "gp");
        if let MessageType::Response(r) = parsed.message_type {
            assert_eq!(r.token.as_deref(), Some("deadbeef"));
            if let Some(super::ResponseType::Values(peers)) = r.response {
                assert_eq!(peers.len(), 1);
                assert_eq!(peers[0].host, "127.0.0.1:6881");
            } else {
                panic!("Expected Values response type")
            }
        } else {
            panic!("Expected Response")
        }
    }

    #[test]
    fn serialize_error() {
        let krpc = KRPC {
            transaction_id: "ee".into(),
            message_type: MessageType::Error(crate::error::DHTError::GenericError),
        };
        let serialized = super::serialize(krpc);
        let parsed = KRPC::deserialize_bytes(serialized.to_vec()).unwrap();
        assert_eq!(parsed.transaction_id, "ee");
        if let MessageType::Error(e) = parsed.message_type {
            matches!(e, crate::error::DHTError::GenericError);
        } else {
            panic!("Expected Error, got: {:?}", parsed.message_type)
        }
    }

    #[test]
    fn deserialize_ping_query() {
        // bencoded: d1:ad2:id20:abcdefghij0123456789e1:q4:ping1:t2:aa1:y1:qe
        let bytes = b"d1:ad2:id20:abcdefghij0123456789e1:q4:ping1:t2:aa1:y1:qe".to_vec();
        let res = KRPC::deserialize_bytes(bytes).unwrap();
        assert_eq!(res.transaction_id, "aa");
        if let MessageType::Query(q) = res.message_type {
            assert!(matches!(q.method_name, super::QueryType::Ping));
            let id = q.get_arg("id").unwrap();
            assert_eq!(id, b"abcdefghij0123456789");
        } else {
            panic!("Expected Query, got: {:?}", res.message_type)
        }
    }

    #[test]
    fn deserialize_find_node_query() {
        let bytes =
            b"d1:ad2:id20:abcdefghij01234567896:target20:mnopqrstuvwxyz123456e1:q9:find_node1:t2:aa1:y1:qe"
                .to_vec();
        let res = KRPC::deserialize_bytes(bytes).unwrap();
        assert_eq!(res.transaction_id, "aa");
        if let MessageType::Query(q) = res.message_type {
            assert!(matches!(q.method_name, super::QueryType::FindNode));
            let id = q.get_arg("id").unwrap();
            assert_eq!(id, b"abcdefghij0123456789");
            let target = q.get_arg("target").unwrap();
            assert_eq!(target, b"mnopqrstuvwxyz123456");
        } else {
            panic!("Expected Query, got: {:?}", res.message_type)
        }
    }

    #[test]
    fn deserialize_get_peers_query() {
        let bytes =
            b"d1:ad2:id20:abcdefghij01234567899:info_hash20:mnopqrstuvwxyz123456e1:q9:get_peers1:t2:aa1:y1:qe"
                .to_vec();
        let res = KRPC::deserialize_bytes(bytes).unwrap();
        if let MessageType::Query(q) = res.message_type {
            assert!(matches!(q.method_name, super::QueryType::FindPeer));
            let info_hash = q.get_arg("info_hash").unwrap();
            assert_eq!(info_hash, b"mnopqrstuvwxyz123456");
        } else {
            panic!("Expected Query, got: {:?}", res.message_type)
        }
    }

    #[test]
    fn deserialize_announce_peer_query() {
        let bytes =
            b"d1:ad2:id20:abcdefghij012345678912:implied_porti1e9:info_hash20:mnopqrstuvwxyz1234564:porti6881e5:token8:aoeusnthe1:q13:announce_peer1:t2:aa1:y1:qe"
                .to_vec();
        let res = KRPC::deserialize_bytes(bytes).unwrap();
        if let MessageType::Query(q) = res.message_type {
            assert!(matches!(q.method_name, super::QueryType::AnnouncePeer));
            let token = q.get_arg("token").unwrap();
            assert_eq!(token, b"aoeusnth");
        } else {
            panic!("Expected Query, got: {:?}", res.message_type)
        }
    }

    #[test]
    fn deserialize_unknown_query_returns_error() {
        let bytes = b"d1:ad2:id20:abcdefghij0123456789e1:q7:unknown1:t2:aa1:y1:qe".to_vec();
        let res = KRPC::deserialize_bytes(bytes);
        assert!(res.is_err());
    }
}
