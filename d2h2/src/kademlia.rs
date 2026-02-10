#![allow(dead_code)]

use crate::error::DHTError;
use crypto::sha1;
use std::time;
use uttd::url::Url;

// we need a few nodes to "bootstrap" our DHT table.
// pub async fn bootstrap() -> Result<(), UttdError> {
// List of other bootstrap nodes:
//
// 1. "router.bittorrent.com:6881" (alias to 2.), *PREFFERED*
// 2. "router.utorrent.com:6881",
// 3. "router.bitcomet.com:6881",
// 4. "dht.transmissionbt.com:6881",
// 5. "dht.aelitis.com:6881",

// use "udp" here, despite not requiring the scheme, to let
// our url library know that we want to create a *UDP* async stream.

// TODO: Complete
async fn bootstrap() {
    unimplemented!()
}

pub fn xor_distance(a: &[u8; 20], b: &[u8; 20]) -> [u8; 20] {
    let mut result = [0u8; 20];
    for i in 0..20 {
        result[i] = a[i] ^ b[i];
    }
    result
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: [u8; 20],
    pub url: Url,
    pub last_seen: time::Instant,
}

#[derive(Debug)]
pub struct DHT {
    pub node_id: [u8; 20],
    pub table: RouteTable,
}

impl DHT {
    pub fn new() -> Result<Self, DHTError> {
        // Generate node_id using two phases
        // BEP_0005 recommends entropy for node id generation which is
        // achieved here by two phase involving current duration in seconds from
        // standard unix time.
        // The duration is multiplied by pseudo-random TinyMT number using the duration
        // as the seed.
        // The result is used to generation a sha1 output, which is the final node_id.
        let mut sha1_gen = sha1::Sha1::new();

        let seed = time::SystemTime::now().duration_since(time::SystemTime::UNIX_EPOCH)?;
        let seed = seed.as_secs() as u32;

        let seed = (crypto::tinymt::TinyMT::rand(seed).get_u32() as u64) * (seed as u64);

        sha1_gen.append_hash(&seed.to_be_bytes());

        let node_id = sha1_gen.get_hash();

        Ok(Self {
            node_id,
            table: RouteTable::new(),
        })
    }
}

const MAX_TABLE_SIZE: usize = 256;

// Routing table
#[derive(Debug)]
pub struct RouteTable {
    nodes: Vec<Node>,
}

impl RouteTable {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn add_node(&mut self, node: Node) {
        // update if already present
        if let Some(existing) = self.nodes.iter_mut().find(|n| n.id == node.id) {
            existing.url = node.url;
            existing.last_seen = node.last_seen;
            return;
        }
        if self.nodes.len() >= MAX_TABLE_SIZE {
            // evict the oldest node
            if let Some(oldest) = self
                .nodes
                .iter()
                .enumerate()
                .min_by_key(|(_, n)| n.last_seen)
                .map(|(i, _)| i)
            {
                self.nodes.swap_remove(oldest);
            }
        }
        self.nodes.push(node);
    }

    pub fn closest_nodes(&self, target: &[u8; 20], count: usize) -> Vec<&Node> {
        let mut sorted: Vec<&Node> = self.nodes.iter().collect();
        sorted.sort_by(|a, b| {
            let dist_a = xor_distance(&a.id, target);
            let dist_b = xor_distance(&b.id, target);
            dist_a.cmp(&dist_b)
        });
        sorted.truncate(count);
        sorted
    }

    pub fn remove_node(&mut self, id: &[u8; 20]) {
        self.nodes.retain(|n| &n.id != id);
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use uttd::url::{Scheme, Url};

    fn make_node(id: [u8; 20]) -> Node {
        Node {
            id,
            url: Url::from_ip_bytes(&[127, 0, 0, 1], 6881, Scheme::UDP),
            last_seen: time::Instant::now(),
        }
    }

    #[test]
    fn xor_distance_identity() {
        let a = [0u8; 20];
        let result = xor_distance(&a, &a);
        assert_eq!(result, [0u8; 20]);
    }

    #[test]
    fn xor_distance_known() {
        let a = [0xFF; 20];
        let b = [0x00; 20];
        let result = xor_distance(&a, &b);
        assert_eq!(result, [0xFF; 20]);
    }

    #[test]
    fn xor_distance_symmetry() {
        let a = [0xAB; 20];
        let b = [0xCD; 20];
        assert_eq!(xor_distance(&a, &b), xor_distance(&b, &a));
    }

    #[test]
    fn route_table_add_and_len() {
        let mut table = RouteTable::new();
        assert_eq!(table.len(), 0);
        table.add_node(make_node([1; 20]));
        assert_eq!(table.len(), 1);
        // duplicate should not increase length
        table.add_node(make_node([1; 20]));
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn route_table_remove() {
        let mut table = RouteTable::new();
        table.add_node(make_node([1; 20]));
        table.add_node(make_node([2; 20]));
        assert_eq!(table.len(), 2);
        table.remove_node(&[1; 20]);
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn route_table_closest_nodes() {
        let mut table = RouteTable::new();
        // target is [0; 20], so node with id closest to 0 should come first
        table.add_node(make_node([0xFF; 20]));
        table.add_node(make_node([0x01; 20]));
        table.add_node(make_node([0x10; 20]));

        let closest = table.closest_nodes(&[0; 20], 2);
        assert_eq!(closest.len(), 2);
        assert_eq!(closest[0].id, [0x01; 20]);
        assert_eq!(closest[1].id, [0x10; 20]);
    }

    #[test]
    fn route_table_evicts_oldest() {
        let mut table = RouteTable::new();
        for i in 0..MAX_TABLE_SIZE {
            let mut id = [0u8; 20];
            id[0] = (i & 0xFF) as u8;
            id[1] = ((i >> 8) & 0xFF) as u8;
            table.add_node(make_node(id));
        }
        assert_eq!(table.len(), MAX_TABLE_SIZE);
        // adding one more should still keep size at MAX_TABLE_SIZE
        table.add_node(make_node([0xFF; 20]));
        assert_eq!(table.len(), MAX_TABLE_SIZE);
    }

    #[test]
    fn dht_new_generates_node_id() {
        let dht = DHT::new().unwrap();
        // node_id should not be all zeros (extremely unlikely with SHA-1)
        assert_ne!(dht.node_id, [0u8; 20]);
    }
}
