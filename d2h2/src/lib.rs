// REFERENCE: https://www.bittorrent.org/beps/bep_0005.html

use std::collections::HashSet;
use std::time;
use std::time::Duration;

use error::D2H2ClientError;
use kademlia::{DHT, Node, RouteTable};
use serde::{MessageType, QueryType, ResponseType, KRPC};
use uttd::url::{Scheme, Url};
use uttd::AsyncStream;

pub mod error;
mod kademlia;
mod request;
pub mod serde;

pub enum PrefferedNetwork {
    UTORRENT,
    Bitorrent,
    Transmission,
    Aelitis,
}

const MAX_LOOKUP_ROUNDS: usize = 8;
// k-closest as per BEP 0005
const K_CLOSEST: usize = 8;
// max DHT response packet size
const DHT_PACKET_SIZE: usize = 1500;
// per-query timeout in seconds
const QUERY_TIMEOUT_SECS: u64 = 5;

#[derive(Debug)]
#[allow(dead_code)]
pub struct D2H2Client {
    node_id: [u8; 20],
    // URL Paths that should be contacted to find peers
    // this vec is updated everytime "k-closest" node is returned from
    // a DHT node instead of peers
    paths: Vec<Url>,
    // number of concurrent requests to send for path
    concurrent: u8,
    table: RouteTable,
}

impl D2H2Client {
    pub fn new(preffered: Option<PrefferedNetwork>) -> Result<Self, D2H2ClientError> {
        let base = match preffered {
            Some(PrefferedNetwork::UTORRENT) => "router.utorrent.com",
            Some(PrefferedNetwork::Transmission) => "dht.transmissionbt.com",
            Some(PrefferedNetwork::Aelitis) => "dht.aelitis.com",
            _ => "router.bittorrent.com",
        };
        let base = Url::from_ip(base, 6881)?;
        let dht = DHT::new()?;
        Ok(Self {
            node_id: dht.node_id,
            paths: vec![base],
            concurrent: 5,
            table: dht.table,
        })
    }

    /// Send a KRPC query to a DHT node and return the parsed response
    async fn send_query(&self, url: &Url, krpc: KRPC) -> Result<KRPC, D2H2ClientError> {
        let target = Url {
            scheme: Scheme::UDP,
            host: url.host.clone(),
            location: "/".to_owned(),
        };
        let mut stream = AsyncStream::new(target).await?;
        let data = serde::serialize(krpc);
        let mut res = vec![0u8; DHT_PACKET_SIZE];
        let read = tokio::time::timeout(
            Duration::from_secs(QUERY_TIMEOUT_SECS),
            stream.send(&data, &mut res),
        )
        .await
        .map_err(|_| D2H2ClientError::LookupTimeout)??;
        res.truncate(read);
        let response = KRPC::deserialize_bytes(res)?;
        Ok(response)
    }

    /// Send a ping query to a DHT node
    pub async fn ping(&self, url: &Url) -> Result<KRPC, D2H2ClientError> {
        let krpc = KRPC::new("pg".into(), QueryType::Ping, &self.node_id);
        self.send_query(url, krpc).await
    }

    /// Iterative DHT lookup for peers matching the given info_hash.
    /// Sends get_peers queries, following closer nodes until peers are found
    /// or no new closer nodes are returned.
    pub async fn find_peers(
        &mut self,
        info_hash: &[u8; 20],
    ) -> Result<Box<[Url]>, D2H2ClientError> {
        let mut peers: Vec<Url> = Vec::new();
        let mut queried: HashSet<String> = HashSet::new();
        // transaction id counter
        let mut tid: u16 = 0;

        // seed the lookup with bootstrap nodes
        let mut to_query: Vec<Url> = self.paths.clone();

        for _ in 0..MAX_LOOKUP_ROUNDS {
            if to_query.is_empty() {
                break;
            }

            let mut next_round: Vec<Url> = Vec::new();

            for url in to_query.drain(..) {
                if queried.contains(&url.host) {
                    continue;
                }
                queried.insert(url.host.clone());

                tid = tid.wrapping_add(1);
                let tid_bytes = tid.to_be_bytes();
                let tid_str =
                    String::from_utf8(tid_bytes.to_vec()).unwrap_or_else(|_| format!("{tid:04x}"));

                let mut krpc =
                    KRPC::new(tid_str, QueryType::FindPeer, &self.node_id);
                // add info_hash to query arguments
                krpc.set_info_hash(info_hash);

                let response = match self.send_query(&url, krpc).await {
                    Ok(r) => r,
                    Err(_) => continue,
                };

                if let MessageType::Response(resp) = response.message_type {
                    match resp.response {
                        Some(ResponseType::Values(found_peers)) => {
                            peers.extend(found_peers.into_vec());
                        }
                        Some(ResponseType::Node(nodes)) => {
                            for node in nodes.iter() {
                                let route_node = Node {
                                    id: node.id,
                                    url: node.node.clone(),
                                    last_seen: time::Instant::now(),
                                };
                                self.table.add_node(route_node);
                                next_round.push(node.node.clone());
                            }
                        }
                        None => {}
                    }
                }
            }

            if !peers.is_empty() {
                break;
            }

            // next round: query the k-closest nodes we know about
            if next_round.is_empty() {
                let closest = self.table.closest_nodes(info_hash, K_CLOSEST);
                for node in closest {
                    if !queried.contains(&node.url.host) {
                        next_round.push(node.url.clone());
                    }
                }
            }

            to_query = next_round;
        }

        Ok(peers.into_boxed_slice())
    }

    /// Send a find_node query to a DHT node
    pub async fn find_node(
        &self,
        url: &Url,
        target: &[u8; 20],
    ) -> Result<KRPC, D2H2ClientError> {
        let mut krpc = KRPC::new("fn".into(), QueryType::FindNode, &self.node_id);
        krpc.set_target(target);
        self.send_query(url, krpc).await
    }
}

#[cfg(test)]
mod test {
    use crate::serde::{MessageType, ResponseType};
    use crate::D2H2Client;
    use uttd::url::Url;

    fn bootstrap_url() -> Url {
        Url::from_ip("router.bittorrent.com", 6881).unwrap()
    }

    // IMPORTANT: These tests interact with real DHT bootstrap nodes on the internet.
    // They may fail if the network is unreachable or the bootstrap nodes are down.

    #[tokio::test]
    async fn ping_bootstrap_node() {
        let client = D2H2Client::new(None).unwrap();
        let url = bootstrap_url();

        let response = client.ping(&url).await.unwrap();

        // a ping response should have a valid transaction id and be a Response type
        assert_eq!(response.transaction_id, "pg");
        if let MessageType::Response(r) = response.message_type {
            // the responding node should include its own id
            // id is 20 raw bytes converted to String via vec_to_string (char-per-byte)
            assert_eq!(r.id.chars().count(), 20);
            // ping responses have no "nodes" or "values"
            assert!(r.response.is_none());
        } else {
            panic!("Expected Response, got: {:?}", response.message_type)
        }
    }

    #[tokio::test]
    async fn ping_multiple_bootstrap_nodes() {
        let client = D2H2Client::new(None).unwrap();

        let nodes = [
            Url::from_ip("router.bittorrent.com", 6881).unwrap(),
            Url::from_ip("router.utorrent.com", 6881).unwrap(),
            Url::from_ip("dht.transmissionbt.com", 6881).unwrap(),
        ];

        let mut responded = 0;
        for url in &nodes {
            if client.ping(url).await.is_ok() {
                responded += 1;
            }
        }
        // at least one bootstrap node should respond
        assert!(responded >= 1, "No bootstrap nodes responded to ping");
    }

    #[tokio::test]
    async fn find_node_returns_nodes() {
        let client = D2H2Client::new(None).unwrap();
        let url = bootstrap_url();

        // use our own node_id as the target to find nodes close to us
        let target = client.node_id;
        let response = client.find_node(&url, &target).await.unwrap();

        assert_eq!(response.transaction_id, "fn");
        if let MessageType::Response(r) = response.message_type {
            assert_eq!(r.id.chars().count(), 20);
            // find_node should return compact node info
            if let Some(ResponseType::Node(nodes)) = r.response {
                assert!(!nodes.is_empty(), "find_node returned no nodes");
                // each node should have a valid 20-byte id and a reachable url
                for node in nodes.iter() {
                    assert_eq!(node.id.len(), 20);
                    assert!(node.node.port() > 0);
                }
            } else {
                panic!("Expected Node response type, got: {:?}", r.response)
            }
        } else {
            panic!("Expected Response, got: {:?}", response.message_type)
        }
    }

    #[tokio::test]
    async fn get_peers_returns_nodes_or_peers() {
        let client = D2H2Client::new(None).unwrap();
        let url = bootstrap_url();

        // use a random info_hash --- bootstrap nodes won't have peers for it,
        // so they should return closer nodes instead
        let info_hash = client.node_id;

        let mut krpc = crate::serde::KRPC::new(
            "gp".into(),
            crate::serde::QueryType::FindPeer,
            &client.node_id,
        );
        krpc.set_info_hash(&info_hash);

        let response = client.send_query(&url, krpc).await.unwrap();

        assert_eq!(response.transaction_id, "gp");
        if let MessageType::Response(r) = response.message_type {
            assert_eq!(r.id.chars().count(), 20);
            match r.response {
                Some(ResponseType::Node(nodes)) => {
                    assert!(!nodes.is_empty(), "get_peers returned empty nodes");
                }
                Some(ResponseType::Values(peers)) => {
                    assert!(!peers.is_empty(), "get_peers returned empty peers");
                }
                None => panic!("get_peers returned no nodes or peers"),
            }
        } else {
            panic!("Expected Response, got: {:?}", response.message_type)
        }
    }

    #[tokio::test]
    async fn find_peers_populates_routing_table() {
        let mut client = D2H2Client::new(None).unwrap();

        // use a random info_hash; the lookup should at least populate
        // the routing table with nodes discovered along the way
        let info_hash = client.node_id;
        let _ = client.find_peers(&info_hash).await;

        assert!(
            client.table.len() > 0,
            "Routing table should have nodes after a find_peers lookup"
        );
    }

    // IMPORTANT: This test depends on a well-seeded torrent having active DHT peers.
    // The debian iso torrent is widely seeded and a good candidate.
    // info_hash for debian-12.8.0-amd64-netinst.iso
    #[tokio::test]
    async fn find_peers_well_known_torrent() {
        let mut client = D2H2Client::new(None).unwrap();

        // SHA-1 info_hash of a well-known debian torrent
        let info_hash: [u8; 20] = [
            0x1b, 0xd0, 0x88, 0xee, 0x91, 0x66, 0xa0, 0x62, 0xcf, 0x4a, 0xf0, 0x9c, 0xf9, 0x97,
            0x20, 0xfa, 0x6e, 0x1a, 0x31, 0x33,
        ];

        let peers = client.find_peers(&info_hash).await.unwrap();

        // we should find at least some peers or at least populate the routing table
        // peers may be empty if the torrent isn't active on DHT, but the table should grow
        if peers.is_empty() {
            assert!(
                client.table.len() > 0,
                "No peers found and routing table is empty — DHT lookup failed entirely"
            );
        } else {
            for peer in peers.iter() {
                assert!(peer.port() > 0);
            }
        }
    }
}
