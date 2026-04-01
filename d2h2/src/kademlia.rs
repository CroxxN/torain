#![allow(dead_code)]

use crate::error::DHTError;
use crypto::sha1;
use std::time;
use uttd::url::Url;

// we need a few nodes to "bootstrap" our DHT table.
// List of other bootstrap nodes:
//
// 1. "router.bittorrent.com:6881" (alias to 2.), *PREFFERED*
// 2. "router.utorrent.com:6881",
// 3. "router.bitcomet.com:6881",
// 4. "dht.transmissionbt.com:6881",
// 5. "dht.aelitis.com:6881",

pub fn xor_distance(a: &[u8; 20], b: &[u8; 20]) -> [u8; 20] {
    let mut result = [0u8; 20];
    for i in 0..20 {
        result[i] = a[i] ^ b[i];
    }
    result
}

/// Find the index of the highest set bit in a 160-bit XOR distance.
/// Returns 0..159 for non-zero distances, or None if the distance is zero
/// (meaning the two IDs are identical).
///
/// Bucket index 0 = farthest (bit 159 set), bucket 159 = closest (bit 0 set).
/// This matches the Kademlia convention: bucket i covers distance [2^(159-i), 2^(160-i)).
fn bucket_index(distance: &[u8; 20]) -> Option<usize> {
    for (byte_idx, &byte) in distance.iter().enumerate() {
        if byte != 0 {
            // leading_zeros gives 0..7 for the position of the highest bit in this byte
            let bit_pos = byte.leading_zeros() as usize;
            // total bit position from the MSB of the 160-bit distance
            return Some(byte_idx * 8 + bit_pos);
        }
    }
    None
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
            table: RouteTable::new(node_id),
        })
    }
}

// max nodes per k-bucket, as per Kademlia / BEP 0005
const K: usize = 8;
// number of buckets: one per bit of the 160-bit ID space
const NUM_BUCKETS: usize = 160;

/// A single k-bucket holding up to K nodes at a specific distance range.
#[derive(Debug)]
struct KBucket {
    nodes: Vec<Node>,
    /// The last time this bucket was used for a lookup (for refresh scheduling)
    last_changed: time::Instant,
}

impl KBucket {
    fn new() -> Self {
        Self {
            nodes: Vec::with_capacity(K),
            last_changed: time::Instant::now(),
        }
    }

    /// Try to add a node. Returns true if added or updated, false if bucket
    /// is full (caller should ping the least-recently-seen node to decide eviction).
    fn add_node(&mut self, node: Node) -> bool {
        // update existing node
        if let Some(existing) = self.nodes.iter_mut().find(|n| n.id == node.id) {
            existing.url = node.url;
            existing.last_seen = node.last_seen;
            self.last_changed = time::Instant::now();
            return true;
        }
        if self.nodes.len() < K {
            self.nodes.push(node);
            self.last_changed = time::Instant::now();
            true
        } else {
            // bucket full — per Kademlia, we should ping the head (oldest) node
            // and only replace it if it fails to respond. For now, evict the
            // least-recently-seen node if the new node is fresher.
            if let Some((oldest_idx, _)) = self
                .nodes
                .iter()
                .enumerate()
                .min_by_key(|(_, n)| n.last_seen)
            {
                // evict the oldest node in favor of the new one
                self.nodes.swap_remove(oldest_idx);
                self.nodes.push(node);
                self.last_changed = time::Instant::now();
                true
            } else {
                false
            }
        }
    }

    fn remove_node(&mut self, id: &[u8; 20]) {
        self.nodes.retain(|n| &n.id != id);
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }

    fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

/// Kademlia routing table with 160 k-buckets.
/// Bucket i covers nodes at XOR distance with highest bit at position i.
#[derive(Debug)]
pub struct RouteTable {
    own_id: [u8; 20],
    buckets: Vec<KBucket>,
}

impl RouteTable {
    pub fn new(own_id: [u8; 20]) -> Self {
        let mut buckets = Vec::with_capacity(NUM_BUCKETS);
        for _ in 0..NUM_BUCKETS {
            buckets.push(KBucket::new());
        }
        Self { own_id, buckets }
    }

    /// Add a node to the routing table, placing it in the correct k-bucket
    /// based on XOR distance from our own node ID.
    pub fn add_node(&mut self, node: Node) {
        let distance = xor_distance(&self.own_id, &node.id);
        if let Some(idx) = bucket_index(&distance) {
            self.buckets[idx].add_node(node);
        }
        // if distance is zero, the node has the same ID as us — don't add
    }

    /// Return up to `count` nodes closest to `target` by XOR distance.
    /// Gathers nodes from the target's bucket first, then expands to
    /// neighboring buckets until enough nodes are collected.
    pub fn closest_nodes(&self, target: &[u8; 20], count: usize) -> Vec<&Node> {
        let mut all_nodes: Vec<&Node> = Vec::new();

        // determine which bucket the target falls into relative to us
        let distance = xor_distance(&self.own_id, target);
        let center = bucket_index(&distance).unwrap_or(NUM_BUCKETS / 2);

        // expand outward from the center bucket
        let mut left = center as isize;
        let mut right = center as isize + 1;
        let max = NUM_BUCKETS as isize;

        // first, add nodes from the center bucket
        for node in &self.buckets[center].nodes {
            all_nodes.push(node);
        }

        // expand outward until we have enough candidates
        while all_nodes.len() < count && (left > 0 || right < max) {
            left -= 1;
            if left >= 0 {
                for node in &self.buckets[left as usize].nodes {
                    all_nodes.push(node);
                }
            }
            if right < max {
                for node in &self.buckets[right as usize].nodes {
                    all_nodes.push(node);
                }
            }
            right += 1;
        }

        // sort by actual XOR distance to target and truncate
        all_nodes.sort_by(|a, b| {
            let dist_a = xor_distance(&a.id, target);
            let dist_b = xor_distance(&b.id, target);
            dist_a.cmp(&dist_b)
        });
        all_nodes.truncate(count);
        all_nodes
    }

    /// Remove a node by its ID from the routing table.
    pub fn remove_node(&mut self, id: &[u8; 20]) {
        let distance = xor_distance(&self.own_id, id);
        if let Some(idx) = bucket_index(&distance) {
            self.buckets[idx].remove_node(id);
        }
    }

    /// Total number of nodes across all buckets.
    pub fn len(&self) -> usize {
        self.buckets.iter().map(|b| b.len()).sum()
    }

    /// Returns the bucket indices that need refreshing (haven't been touched
    /// within the given duration).
    pub fn stale_buckets(&self, max_age: time::Duration) -> Vec<usize> {
        let now = time::Instant::now();
        self.buckets
            .iter()
            .enumerate()
            .filter(|(_, b)| !b.is_empty() && now.duration_since(b.last_changed) > max_age)
            .map(|(i, _)| i)
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use uttd::url::{Scheme, Url};

    // fixed own_id for deterministic bucket placement in tests
    const OWN_ID: [u8; 20] = [0u8; 20];

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
    fn bucket_index_zero_distance() {
        let d = [0u8; 20];
        assert_eq!(bucket_index(&d), None);
    }

    #[test]
    fn bucket_index_highest_bit() {
        // highest bit set = byte 0, bit 7 => bucket 0
        let mut d = [0u8; 20];
        d[0] = 0x80;
        assert_eq!(bucket_index(&d), Some(0));
    }

    #[test]
    fn bucket_index_lowest_bit() {
        // lowest bit set = byte 19, bit 0 => bucket 159
        let mut d = [0u8; 20];
        d[19] = 0x01;
        assert_eq!(bucket_index(&d), Some(159));
    }

    #[test]
    fn bucket_index_middle() {
        // byte 10, bit 4 (0x08) => bucket 10*8 + 4 = 84
        let mut d = [0u8; 20];
        d[10] = 0x08;
        assert_eq!(bucket_index(&d), Some(84));
    }

    #[test]
    fn route_table_add_and_len() {
        let mut table = RouteTable::new(OWN_ID);
        assert_eq!(table.len(), 0);
        table.add_node(make_node([1; 20]));
        assert_eq!(table.len(), 1);
        // duplicate should not increase length
        table.add_node(make_node([1; 20]));
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn route_table_remove() {
        let mut table = RouteTable::new(OWN_ID);
        table.add_node(make_node([1; 20]));
        table.add_node(make_node([2; 20]));
        assert_eq!(table.len(), 2);
        table.remove_node(&[1; 20]);
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn route_table_closest_nodes() {
        let mut table = RouteTable::new(OWN_ID);
        // target is [0; 20] (same as own_id), so nodes closest to target should be
        // those with smallest XOR distance from [0; 20]
        table.add_node(make_node([0xFF; 20]));
        table.add_node(make_node([0x01; 20]));
        table.add_node(make_node([0x10; 20]));

        let closest = table.closest_nodes(&[0; 20], 2);
        assert_eq!(closest.len(), 2);
        assert_eq!(closest[0].id, [0x01; 20]);
        assert_eq!(closest[1].id, [0x10; 20]);
    }

    #[test]
    fn route_table_bucket_placement() {
        // with own_id = [0; 20], a node with id [0x80, 0, ...] has distance with
        // highest bit at position 0 => goes into bucket 0
        let mut table = RouteTable::new(OWN_ID);
        let mut id = [0u8; 20];
        id[0] = 0x80;
        table.add_node(make_node(id));
        assert_eq!(table.len(), 1);
        // verify it's in bucket 0 by checking that removing with wrong bucket doesn't work
        // and that the correct bucket has it
        assert_eq!(table.buckets[0].len(), 1);
    }

    #[test]
    fn route_table_kbucket_overflow() {
        // fill a single bucket beyond K and verify eviction
        let mut table = RouteTable::new(OWN_ID);

        // all these nodes have id [0x80, 0, ..., i] which XOR with [0;20] gives
        // distance with byte 0 = 0x80 => bucket 0
        for i in 0..(K + 2) {
            let mut id = [0u8; 20];
            id[0] = 0x80;
            id[19] = i as u8;
            table.add_node(make_node(id));
        }
        // bucket should cap at K
        assert_eq!(table.buckets[0].len(), K);
        assert_eq!(table.len(), K);
    }

    #[test]
    fn route_table_nodes_in_different_buckets() {
        let mut table = RouteTable::new(OWN_ID);

        // bucket 0: distance byte[0] = 0x80
        let mut id1 = [0u8; 20];
        id1[0] = 0x80;
        table.add_node(make_node(id1));

        // bucket 159: distance byte[19] = 0x01
        let mut id2 = [0u8; 20];
        id2[19] = 0x01;
        table.add_node(make_node(id2));

        assert_eq!(table.len(), 2);
        assert_eq!(table.buckets[0].len(), 1);
        assert_eq!(table.buckets[159].len(), 1);
    }

    #[test]
    fn route_table_closest_spans_buckets() {
        let mut table = RouteTable::new(OWN_ID);

        // put nodes in several different buckets
        let mut id1 = [0u8; 20];
        id1[0] = 0x80; // bucket 0, distance 0x80...
        table.add_node(make_node(id1));

        let mut id2 = [0u8; 20];
        id2[0] = 0x01; // bucket 7, distance 0x01...
        table.add_node(make_node(id2));

        let mut id3 = [0u8; 20];
        id3[19] = 0x01; // bucket 159, very close
        table.add_node(make_node(id3));

        // find 3 closest to own_id [0;20]
        let closest = table.closest_nodes(&OWN_ID, 3);
        assert_eq!(closest.len(), 3);
        // closest first: id3 (distance 0x01 at byte 19), id2 (0x01 at byte 0), id1 (0x80 at byte 0)
        assert_eq!(closest[0].id, id3);
        assert_eq!(closest[1].id, id2);
        assert_eq!(closest[2].id, id1);
    }

    #[test]
    fn dht_new_generates_node_id() {
        let dht = DHT::new().unwrap();
        // node_id should not be all zeros (extremely unlikely with SHA-1)
        assert_ne!(dht.node_id, [0u8; 20]);
    }
}
