#![allow(dead_code)]

use std::collections::HashSet;
use std::net::SocketAddr;
use std::time::{self, Duration};

use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use crate::error::D2H2ClientError;
use crate::handler;
use crate::kademlia::{DHT, Node, RouteTable};
use crate::serde::{self, MessageType, QueryType, ResponseType, KRPC};
use crate::store::PeerStore;
use crate::token::TokenManager;
use uttd::url::{Scheme, Url};

/// Default listen port for the DHT server
const DEFAULT_PORT: u16 = 6881;
/// Max UDP packet size
const DHT_PACKET_SIZE: usize = 1500;
/// Timeout for individual queries (seconds)
const QUERY_TIMEOUT_SECS: u64 = 5;
/// k-closest as per BEP 0005
const K_CLOSEST: usize = 8;
/// Max iterative lookup rounds
const MAX_LOOKUP_ROUNDS: usize = 8;
/// Bucket refresh interval: 15 minutes
const BUCKET_REFRESH_SECS: u64 = 15 * 60;
/// Peer store eviction interval: 5 minutes
const STORE_EVICTION_SECS: u64 = 5 * 60;

/// Bootstrap router addresses
const BOOTSTRAP_NODES: &[(&str, u16)] = &[
    ("router.bittorrent.com", 6881),
    ("router.utorrent.com", 6881),
    ("dht.transmissionbt.com", 6881),
];

/// A request from the application layer to the D2H2 server
#[derive(Debug)]
pub enum D2H2Request {
    /// Find peers for the given info_hash, reply on the oneshot channel
    FindPeers {
        info_hash: [u8; 20],
        reply: mpsc::Sender<Vec<Url>>,
    },
    /// Announce that we have the given info_hash on the given port
    AnnouncePeer { info_hash: [u8; 20], port: u16 },
    /// Shut down the server
    Shutdown,
}

/// The D2H2 DHT server. Runs as a long-lived task, handling both:
/// - Incoming KRPC queries from other DHT nodes (server mode)
/// - Outgoing lookups requested by the application (client mode)
pub struct D2H2Server {
    node_id: [u8; 20],
    table: RouteTable,
    store: PeerStore,
    token_mgr: TokenManager,
    socket: UdpSocket,
    /// Transaction ID counter for outgoing queries
    tid: u16,
}

impl D2H2Server {
    /// Bind a UDP socket and initialize the DHT server.
    /// Call `run()` to start the event loop.
    pub async fn bind(port: u16) -> Result<Self, D2H2ClientError> {
        let dht = DHT::new()?;
        let addr = format!("0.0.0.0:{}", port);
        let socket = UdpSocket::bind(&addr).await.map_err(|e| {
            D2H2ClientError::NetworkError(uttd::UttdError::IoError(e))
        })?;

        Ok(Self {
            node_id: dht.node_id,
            table: dht.table,
            store: PeerStore::new(),
            token_mgr: TokenManager::new(),
            socket,
            tid: 0,
        })
    }

    /// Get the node ID of this server
    pub fn node_id(&self) -> &[u8; 20] {
        &self.node_id
    }

    /// Generate the next transaction ID as a 2-byte string
    fn next_tid(&mut self) -> String {
        self.tid = self.tid.wrapping_add(1);
        let bytes = self.tid.to_be_bytes();
        String::from_utf8(bytes.to_vec()).unwrap_or_else(|_| format!("{:04x}", self.tid))
    }

    /// Send a KRPC query to a specific address and return the raw response bytes.
    async fn send_query_to(
        &self,
        addr: &SocketAddr,
        krpc: KRPC,
    ) -> Result<Vec<u8>, D2H2ClientError> {
        let data = serde::serialize(krpc);
        self.socket.send_to(&data, addr).await.map_err(|e| {
            D2H2ClientError::NetworkError(uttd::UttdError::IoError(e))
        })?;

        let mut buf = vec![0u8; DHT_PACKET_SIZE];
        let (len, _) = tokio::time::timeout(
            Duration::from_secs(QUERY_TIMEOUT_SECS),
            self.socket.recv_from(&mut buf),
        )
        .await
        .map_err(|_| D2H2ClientError::LookupTimeout)?
        .map_err(|e| D2H2ClientError::NetworkError(uttd::UttdError::IoError(e)))?;

        buf.truncate(len);
        Ok(buf)
    }

    /// Resolve a Url to a SocketAddr
    fn url_to_addr(url: &Url) -> Option<SocketAddr> {
        url.host.parse().ok()
    }

    /// Bootstrap: populate the routing table by finding our own node ID
    /// through the bootstrap nodes.
    pub async fn bootstrap(&mut self) -> Result<(), D2H2ClientError> {
        let mut queried: HashSet<String> = HashSet::new();
        let mut to_query: Vec<SocketAddr> = Vec::new();

        // seed with bootstrap nodes
        for &(host, port) in BOOTSTRAP_NODES {
            if let Ok(url) = Url::from_ip(host, port) {
                if let Some(addr) = Self::url_to_addr(&url) {
                    to_query.push(addr);
                }
            }
        }

        // resolve bootstrap hostnames via DNS
        for &(host, port) in BOOTSTRAP_NODES {
            let addr_str = format!("{}:{}", host, port);
            let resolved: Vec<SocketAddr> = tokio::net::lookup_host(&addr_str)
                .await
                .map(|addrs| addrs.collect())
                .unwrap_or_default();
            to_query.extend(resolved);
        }

        // iterative find_node for our own ID
        for _ in 0..MAX_LOOKUP_ROUNDS {
            if to_query.is_empty() {
                break;
            }

            let mut next_round: Vec<SocketAddr> = Vec::new();

            for addr in to_query.drain(..) {
                let addr_str = addr.to_string();
                if queried.contains(&addr_str) {
                    continue;
                }
                queried.insert(addr_str);

                let tid = self.next_tid();
                let mut krpc = KRPC::new(tid, QueryType::FindNode, &self.node_id);
                krpc.set_target(&self.node_id);

                let response_bytes = match self.send_query_to(&addr, krpc).await {
                    Ok(r) => r,
                    Err(_) => continue,
                };

                let response = match KRPC::deserialize_bytes(response_bytes) {
                    Ok(r) => r,
                    Err(_) => continue,
                };

                if let MessageType::Response(resp) = response.message_type {
                    if let Some(ResponseType::Node(nodes)) = resp.response {
                        for node in nodes.iter() {
                            let route_node = Node {
                                id: node.id,
                                url: node.node.clone(),
                                last_seen: time::Instant::now(),
                            };
                            self.table.add_node(route_node);
                            if let Some(a) = Self::url_to_addr(&node.node) {
                                if !queried.contains(&a.to_string()) {
                                    next_round.push(a);
                                }
                            }
                        }
                    }
                }
            }

            // also check our table for closer nodes
            if next_round.is_empty() {
                let closest = self.table.closest_nodes(&self.node_id, K_CLOSEST);
                for node in closest {
                    if let Some(a) = Self::url_to_addr(&node.url) {
                        if !queried.contains(&a.to_string()) {
                            next_round.push(a);
                        }
                    }
                }
            }

            to_query = next_round;
        }

        Ok(())
    }

    /// Run the DHT server event loop. Processes:
    /// - Incoming KRPC queries from other nodes
    /// - Outgoing requests from the application via the request channel
    /// - Periodic maintenance (bucket refresh, peer store eviction)
    pub async fn run(
        &mut self,
        mut request_rx: mpsc::Receiver<D2H2Request>,
    ) -> Result<(), D2H2ClientError> {
        let mut buf = vec![0u8; DHT_PACKET_SIZE];
        let mut refresh_interval = tokio::time::interval(Duration::from_secs(BUCKET_REFRESH_SECS));
        let mut eviction_interval = tokio::time::interval(Duration::from_secs(STORE_EVICTION_SECS));

        loop {
            tokio::select! {
                // handle incoming UDP packets
                result = self.socket.recv_from(&mut buf) => {
                    match result {
                        Ok((len, src_addr)) => {
                            let packet = buf[..len].to_vec();
                            self.handle_incoming_packet(&packet, &src_addr).await;
                        }
                        Err(_) => continue,
                    }
                }

                // handle application requests
                request = request_rx.recv() => {
                    match request {
                        Some(D2H2Request::FindPeers { info_hash, reply }) => {
                            let peers = self.find_peers(&info_hash).await;
                            let _ = reply.send(peers).await;
                        }
                        Some(D2H2Request::AnnouncePeer { info_hash, port }) => {
                            self.announce_to_closest(&info_hash, port).await;
                        }
                        Some(D2H2Request::Shutdown) | None => {
                            break;
                        }
                    }
                }

                // periodic bucket refresh
                _ = refresh_interval.tick() => {
                    self.refresh_stale_buckets().await;
                }

                // periodic peer store eviction
                _ = eviction_interval.tick() => {
                    self.store.evict_expired();
                }
            }
        }

        Ok(())
    }

    /// Handle an incoming UDP packet: parse it, handle if query, ignore otherwise
    async fn handle_incoming_packet(&mut self, packet: &[u8], src_addr: &SocketAddr) {
        let krpc = match KRPC::deserialize_bytes(packet.to_vec()) {
            Ok(k) => k,
            Err(_) => return,
        };

        // only handle queries from other nodes
        if let MessageType::Query(_) = &krpc.message_type {
            let ip_bytes: Vec<u8> = match src_addr {
                SocketAddr::V4(v4) => v4.ip().octets().to_vec(),
                SocketAddr::V6(v6) => v6.ip().octets().to_vec(),
            };
            let requester_url = Url {
                scheme: Scheme::UDP,
                host: src_addr.to_string(),
                location: "/".to_owned(),
            };

            // update routing table with the querying node
            if let MessageType::Query(ref q) = krpc.message_type {
                if let Some(id_bytes) = q.get_arg("id") {
                    if id_bytes.len() == 20 {
                        let mut id = [0u8; 20];
                        id.copy_from_slice(id_bytes);
                        self.table.add_node(Node {
                            id,
                            url: requester_url.clone(),
                            last_seen: time::Instant::now(),
                        });
                    }
                }
            }

            let response = handler::handle_query(
                &krpc,
                &self.node_id,
                &self.table,
                &mut self.store,
                &mut self.token_mgr,
                &ip_bytes,
                &requester_url,
            );

            let data = serde::serialize(response);
            let _ = self.socket.send_to(&data, src_addr).await;
        }
    }

    /// Iterative DHT lookup for peers matching the given info_hash
    async fn find_peers(&mut self, info_hash: &[u8; 20]) -> Vec<Url> {
        let mut peers: Vec<Url> = Vec::new();
        let mut queried: HashSet<String> = HashSet::new();

        // seed from routing table or bootstrap nodes
        let mut to_query: Vec<SocketAddr> = self
            .table
            .closest_nodes(info_hash, K_CLOSEST)
            .iter()
            .filter_map(|n| Self::url_to_addr(&n.url))
            .collect();

        if to_query.is_empty() {
            // fall back to bootstrap
            for &(host, port) in BOOTSTRAP_NODES {
                let addr_str = format!("{}:{}", host, port);
                let resolved: Vec<SocketAddr> = tokio::net::lookup_host(&addr_str)
                    .await
                    .map(|addrs| addrs.collect())
                    .unwrap_or_default();
                to_query.extend(resolved);
            }
        }

        for _ in 0..MAX_LOOKUP_ROUNDS {
            if to_query.is_empty() {
                break;
            }

            let mut next_round: Vec<SocketAddr> = Vec::new();

            for addr in to_query.drain(..) {
                let addr_str = addr.to_string();
                if queried.contains(&addr_str) {
                    continue;
                }
                queried.insert(addr_str);

                let tid = self.next_tid();
                let mut krpc = KRPC::new(tid, QueryType::FindPeer, &self.node_id);
                krpc.set_info_hash(info_hash);

                let response_bytes = match self.send_query_to(&addr, krpc).await {
                    Ok(r) => r,
                    Err(_) => continue,
                };

                let response = match KRPC::deserialize_bytes(response_bytes) {
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
                                if let Some(a) = Self::url_to_addr(&node.node) {
                                    if !queried.contains(&a.to_string()) {
                                        next_round.push(a);
                                    }
                                }
                            }
                        }
                        None => {}
                    }
                }
            }

            if !peers.is_empty() {
                break;
            }

            if next_round.is_empty() {
                let closest = self.table.closest_nodes(info_hash, K_CLOSEST);
                for node in closest {
                    if let Some(a) = Self::url_to_addr(&node.url) {
                        if !queried.contains(&a.to_string()) {
                            next_round.push(a);
                        }
                    }
                }
            }

            to_query = next_round;
        }

        peers
    }

    /// Announce to the K closest nodes that we have a torrent
    async fn announce_to_closest(&mut self, info_hash: &[u8; 20], port: u16) {
        let closest = self.table.closest_nodes(info_hash, K_CLOSEST);
        let addrs: Vec<(SocketAddr, Option<String>)> = closest
            .iter()
            .filter_map(|n| {
                Self::url_to_addr(&n.url).map(|a| (a, None))
            })
            .collect();

        // first do get_peers to get tokens, then announce
        for (addr, _) in &addrs {
            let tid = self.next_tid();
            let mut krpc = KRPC::new(tid, QueryType::FindPeer, &self.node_id);
            krpc.set_info_hash(info_hash);

            if let Ok(response_bytes) = self.send_query_to(addr, krpc).await {
                if let Ok(response) = KRPC::deserialize_bytes(response_bytes) {
                    if let MessageType::Response(resp) = response.message_type {
                        if let Some(token) = resp.token {
                            // now send announce_peer with the token
                            let tid = self.next_tid();
                            let mut ap = KRPC::new(tid, QueryType::AnnouncePeer, &self.node_id);
                            ap.set_info_hash(info_hash);
                            ap.set_port(port);
                            ap.set_token(&token);
                            ap.set_implied_port(false);
                            let _ = self.send_query_to(addr, ap).await;
                        }
                    }
                }
            }
        }
    }

    /// Refresh stale buckets by doing a find_node for a random ID in each stale bucket's range
    async fn refresh_stale_buckets(&mut self) {
        let stale = self.table.stale_buckets(Duration::from_secs(BUCKET_REFRESH_SECS));
        for bucket_idx in stale {
            // generate a target ID that falls into this bucket
            let mut target = self.node_id;
            // flip the bit at position bucket_idx to ensure the target falls in that bucket
            let byte_idx = bucket_idx / 8;
            let bit_idx = 7 - (bucket_idx % 8);
            target[byte_idx] ^= 1 << bit_idx;

            // do a find_node for this target against our closest known nodes
            let addrs: Vec<SocketAddr> = self
                .table
                .closest_nodes(&target, K_CLOSEST)
                .iter()
                .filter_map(|n| Self::url_to_addr(&n.url))
                .collect();

            for addr in addrs {
                let tid = self.next_tid();
                let mut krpc = KRPC::new(tid, QueryType::FindNode, &self.node_id);
                krpc.set_target(&target);

                if let Ok(response_bytes) = self.send_query_to(&addr, krpc).await {
                    if let Ok(response) = KRPC::deserialize_bytes(response_bytes) {
                        if let MessageType::Response(resp) = response.message_type {
                            if let Some(ResponseType::Node(nodes)) = resp.response {
                                for n in nodes.iter() {
                                    self.table.add_node(Node {
                                        id: n.id,
                                        url: n.node.clone(),
                                        last_seen: time::Instant::now(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Spawn a D2H2 server as a background tokio task.
/// Returns a channel sender for sending requests to the server.
pub async fn spawn_d2h2(port: u16) -> Result<mpsc::Sender<D2H2Request>, D2H2ClientError> {
    let mut server = D2H2Server::bind(port).await?;

    // bootstrap
    server.bootstrap().await?;

    let (tx, rx) = mpsc::channel(64);

    tokio::spawn(async move {
        let _ = server.run(rx).await;
    });

    Ok(tx)
}
