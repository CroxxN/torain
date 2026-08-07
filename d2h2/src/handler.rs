#![allow(dead_code)]

use crate::kademlia::RouteTable;
use crate::serde::{MessageType, QueryType, Response, ResponseNode, ResponseType, KRPC};
use crate::store::PeerStore;
use crate::token::TokenManager;
use uttd::url::{Scheme, Url};

// k-closest as per BEP 0005
const K_CLOSEST: usize = 8;

/// Handle an incoming KRPC query and produce the appropriate response.
/// The caller is responsible for serializing the response and sending it back.
///
/// Arguments:
///   - query: the parsed incoming KRPC (must be MessageType::Query)
///   - own_id: our 20-byte node ID
///   - table: our routing table (for find_node / get_peers fallback)
///   - store: our peer store (for get_peers / announce_peer)
///   - token_mgr: token manager for get_peers / announce_peer
///   - requester_ip: IP bytes of the requester (for token generation)
///   - requester_url: full URL of the requester (for announce_peer storage)
pub fn handle_query(
    query: &KRPC,
    own_id: &[u8; 20],
    table: &RouteTable,
    store: &mut PeerStore,
    token_mgr: &mut TokenManager,
    requester_ip: &[u8],
    requester_url: &Url,
) -> KRPC {
    let q = match &query.message_type {
        MessageType::Query(q) => q,
        _ => {
            // not a query — return a protocol error
            return KRPC {
                transaction_id: query.transaction_id.clone(),
                message_type: MessageType::Error(crate::error::DHTError::ProtocError),
            };
        }
    };

    // extract the querying node's ID and update the routing table knowledge
    // (the actual table update is done by the caller to avoid borrow issues)
    let id_str = own_id_string(own_id);

    match q.method_name {
        QueryType::Ping => handle_ping(&query.transaction_id, &id_str),

        QueryType::FindNode => {
            let target = match q.get_arg("target") {
                Some(t) if t.len() == 20 => {
                    let mut arr = [0u8; 20];
                    arr.copy_from_slice(t);
                    arr
                }
                _ => {
                    return KRPC {
                        transaction_id: query.transaction_id.clone(),
                        message_type: MessageType::Error(crate::error::DHTError::ProtocError),
                    };
                }
            };
            handle_find_node(&query.transaction_id, &id_str, table, &target)
        }

        QueryType::FindPeer => {
            let info_hash = match q.get_arg("info_hash") {
                Some(h) if h.len() == 20 => {
                    let mut arr = [0u8; 20];
                    arr.copy_from_slice(h);
                    arr
                }
                _ => {
                    return KRPC {
                        transaction_id: query.transaction_id.clone(),
                        message_type: MessageType::Error(crate::error::DHTError::ProtocError),
                    };
                }
            };
            handle_get_peers(
                &query.transaction_id,
                &id_str,
                table,
                store,
                token_mgr,
                requester_ip,
                &info_hash,
            )
        }

        QueryType::AnnouncePeer => {
            let info_hash = match q.get_arg("info_hash") {
                Some(h) if h.len() == 20 => {
                    let mut arr = [0u8; 20];
                    arr.copy_from_slice(h);
                    arr
                }
                _ => {
                    return KRPC {
                        transaction_id: query.transaction_id.clone(),
                        message_type: MessageType::Error(crate::error::DHTError::ProtocError),
                    };
                }
            };
            let token = match q.get_arg("token") {
                Some(t) => t,
                None => {
                    return KRPC {
                        transaction_id: query.transaction_id.clone(),
                        message_type: MessageType::Error(crate::error::DHTError::ProtocError),
                    };
                }
            };
            handle_announce_peer(
                &query.transaction_id,
                &id_str,
                store,
                token_mgr,
                requester_ip,
                requester_url,
                &info_hash,
                token,
                q.get_arg("port"),
                q.get_arg("implied_port"),
            )
        }
    }
}

fn own_id_string(id: &[u8; 20]) -> String {
    id.iter().map(|&b| b as char).collect()
}

fn handle_ping(tid: &str, own_id: &str) -> KRPC {
    let resp = Response::new(own_id.to_string(), None, None);
    KRPC {
        transaction_id: tid.to_string(),
        message_type: MessageType::Response(resp),
    }
}

fn handle_find_node(tid: &str, own_id: &str, table: &RouteTable, target: &[u8; 20]) -> KRPC {
    let closest = table.closest_nodes(target, K_CLOSEST);
    let nodes: Vec<ResponseNode> = closest
        .into_iter()
        .map(|n| ResponseNode {
            id: n.id,
            node: n.url.clone(),
        })
        .collect();

    let resp_type = if nodes.is_empty() {
        None
    } else {
        Some(ResponseType::Node(nodes.into()))
    };

    let resp = Response::new(own_id.to_string(), None, resp_type);
    KRPC {
        transaction_id: tid.to_string(),
        message_type: MessageType::Response(resp),
    }
}

fn handle_get_peers(
    tid: &str,
    own_id: &str,
    table: &RouteTable,
    store: &PeerStore,
    token_mgr: &mut TokenManager,
    requester_ip: &[u8],
    info_hash: &[u8; 20],
) -> KRPC {
    let token_bytes = token_mgr.generate(requester_ip);
    let token_str: String = token_bytes.iter().map(|&b| b as char).collect();

    // check if we have peers for this info_hash
    if let Some(peers) = store.get_peers(info_hash) {
        let urls: Vec<Url> = peers.into_iter().cloned().collect();
        let resp = Response::new(
            own_id.to_string(),
            Some(token_str),
            Some(ResponseType::Values(urls.into())),
        );
        return KRPC {
            transaction_id: tid.to_string(),
            message_type: MessageType::Response(resp),
        };
    }

    // no peers known — return closest nodes
    let closest = table.closest_nodes(info_hash, K_CLOSEST);
    let nodes: Vec<ResponseNode> = closest
        .into_iter()
        .map(|n| ResponseNode {
            id: n.id,
            node: n.url.clone(),
        })
        .collect();

    let resp_type = if nodes.is_empty() {
        None
    } else {
        Some(ResponseType::Node(nodes.into()))
    };

    let resp = Response::new(own_id.to_string(), Some(token_str), resp_type);
    KRPC {
        transaction_id: tid.to_string(),
        message_type: MessageType::Response(resp),
    }
}

fn handle_announce_peer(
    tid: &str,
    own_id: &str,
    store: &mut PeerStore,
    token_mgr: &mut TokenManager,
    requester_ip: &[u8],
    requester_url: &Url,
    info_hash: &[u8; 20],
    token: &[u8],
    port_arg: Option<&[u8]>,
    implied_port_arg: Option<&[u8]>,
) -> KRPC {
    // validate token
    if !token_mgr.validate(token, requester_ip) {
        return KRPC {
            transaction_id: tid.to_string(),
            message_type: MessageType::Error(crate::error::DHTError::ProtocError),
        };
    }

    // determine the port to store
    let implied = implied_port_arg
        .map(|b| !b.is_empty() && b[0] != 0)
        .unwrap_or(false);

    let peer_url = if implied {
        // use the source port of the requester
        requester_url.clone()
    } else if let Some(port_bytes) = port_arg {
        // parse the port from the argument
        let port = if port_bytes.len() >= 2 {
            u16::from_be_bytes([port_bytes[0], port_bytes[1]])
        } else if port_bytes.len() == 1 {
            port_bytes[0] as u16
        } else {
            requester_url.port()
        };
        // build a new URL with the announced port
        let (ip_str, _) = requester_url
            .host
            .rsplit_once(':')
            .unwrap_or((&requester_url.host, "0"));
        Url {
            scheme: Scheme::UDP,
            host: format!("{}:{}", ip_str, port),
            location: "/".to_owned(),
        }
    } else {
        requester_url.clone()
    };

    store.add_peer(info_hash, peer_url);

    // announce_peer response is just an ack with our id
    let resp = Response::new(own_id.to_string(), None, None);
    KRPC {
        transaction_id: tid.to_string(),
        message_type: MessageType::Response(resp),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::kademlia::RouteTable;
    use crate::serde::{MessageType, QueryType, KRPC};
    use crate::store::PeerStore;
    use crate::token::TokenManager;
    use std::time;
    use uttd::url::{Scheme, Url};

    fn own_id() -> [u8; 20] {
        [0xAA; 20]
    }

    fn requester_url() -> Url {
        Url::from_ip_bytes(&[192, 168, 1, 10], 6881, Scheme::UDP)
    }

    fn requester_ip() -> Vec<u8> {
        vec![192, 168, 1, 10]
    }

    fn make_ping_query() -> KRPC {
        KRPC::new("pg".into(), QueryType::Ping, &[0xBB; 20])
    }

    fn make_find_node_query(target: &[u8; 20]) -> KRPC {
        let mut krpc = KRPC::new("fn".into(), QueryType::FindNode, &[0xBB; 20]);
        krpc.set_target(target);
        krpc
    }

    fn make_get_peers_query(info_hash: &[u8; 20]) -> KRPC {
        let mut krpc = KRPC::new("gp".into(), QueryType::FindPeer, &[0xBB; 20]);
        krpc.set_info_hash(info_hash);
        krpc
    }

    #[test]
    fn handle_ping_returns_response() {
        let id = own_id();
        let table = RouteTable::new(id);
        let mut store = PeerStore::new();
        let mut tokens = TokenManager::new();
        let query = make_ping_query();

        let resp = handle_query(
            &query,
            &id,
            &table,
            &mut store,
            &mut tokens,
            &requester_ip(),
            &requester_url(),
        );

        assert_eq!(resp.transaction_id, "pg");
        if let MessageType::Response(r) = resp.message_type {
            assert_eq!(r.id.chars().count(), 20);
            assert!(r.response.is_none());
        } else {
            panic!("Expected Response")
        }
    }

    #[test]
    fn handle_find_node_returns_nodes() {
        let id = own_id();
        let mut table = RouteTable::new(id);

        // add some nodes to the table
        for i in 0..5u8 {
            let mut nid = [0u8; 20];
            nid[0] = i + 1;
            table.add_node(crate::kademlia::Node {
                id: nid,
                url: Url::from_ip_bytes(&[127, 0, 0, i + 1], 6881, Scheme::UDP),
                last_seen: time::Instant::now(),
            });
        }

        let mut store = PeerStore::new();
        let mut tokens = TokenManager::new();
        let target = [0x01; 20];
        let query = make_find_node_query(&target);

        let resp = handle_query(
            &query,
            &id,
            &table,
            &mut store,
            &mut tokens,
            &requester_ip(),
            &requester_url(),
        );

        if let MessageType::Response(r) = resp.message_type {
            if let Some(ResponseType::Node(nodes)) = r.response {
                assert!(!nodes.is_empty());
            } else {
                panic!("Expected Node response type")
            }
        } else {
            panic!("Expected Response")
        }
    }

    #[test]
    fn handle_get_peers_returns_token() {
        let id = own_id();
        let table = RouteTable::new(id);
        let mut store = PeerStore::new();
        let mut tokens = TokenManager::new();
        let info_hash = [0xDD; 20];
        let query = make_get_peers_query(&info_hash);

        let resp = handle_query(
            &query,
            &id,
            &table,
            &mut store,
            &mut tokens,
            &requester_ip(),
            &requester_url(),
        );

        if let MessageType::Response(r) = resp.message_type {
            assert!(
                r.token.is_some(),
                "get_peers response should include a token"
            );
        } else {
            panic!("Expected Response")
        }
    }

    #[test]
    fn handle_get_peers_returns_peers_when_known() {
        let id = own_id();
        let table = RouteTable::new(id);
        let mut store = PeerStore::new();
        let mut tokens = TokenManager::new();

        let info_hash = [0xEE; 20];
        store.add_peer(
            &info_hash,
            Url::from_ip_bytes(&[10, 0, 0, 1], 51413, Scheme::UDP),
        );

        let query = make_get_peers_query(&info_hash);
        let resp = handle_query(
            &query,
            &id,
            &table,
            &mut store,
            &mut tokens,
            &requester_ip(),
            &requester_url(),
        );

        if let MessageType::Response(r) = resp.message_type {
            if let Some(ResponseType::Values(peers)) = r.response {
                assert_eq!(peers.len(), 1);
                assert_eq!(peers[0].port(), 51413);
            } else {
                panic!("Expected Values response type, got {:?}", r.response)
            }
        } else {
            panic!("Expected Response")
        }
    }

    #[test]
    fn handle_announce_peer_stores_peer() {
        let id = own_id();
        let table = RouteTable::new(id);
        let mut store = PeerStore::new();
        let mut tokens = TokenManager::new();
        let ip = requester_ip();
        let url = requester_url();

        let info_hash = [0xFF; 20];

        // first get a valid token
        let token = tokens.generate(&ip);

        // build an announce_peer query
        let mut query = KRPC::new("ap".into(), QueryType::AnnouncePeer, &[0xBB; 20]);
        query.set_info_hash(&info_hash);
        query.set_port(51413);
        // set token as raw bytes to match what TokenManager generates
        query.set_token_bytes(&token);
        query.set_implied_port(true);

        let resp = handle_query(&query, &id, &table, &mut store, &mut tokens, &ip, &url);

        if let MessageType::Response(r) = resp.message_type {
            assert!(r.response.is_none()); // announce_peer returns just id
        } else {
            panic!("Expected Response, got: {:?}", resp.message_type)
        }

        // verify peer was stored
        let peers = store.get_peers(&info_hash);
        assert!(peers.is_some());
    }
}
