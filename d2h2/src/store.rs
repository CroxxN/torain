#![allow(dead_code)]

use std::collections::HashMap;
use std::time;
use uttd::url::Url;

/// TTL for stored peers: 30 minutes (standard for DHT)
const PEER_TTL_SECS: u64 = 30 * 60;

/// Entry for a single peer announcing on an info_hash
#[derive(Debug, Clone)]
struct PeerEntry {
    url: Url,
    added: time::Instant,
}

/// Storage for info_hash -> peer mappings received via announce_peer.
/// Peers expire after PEER_TTL_SECS.
#[derive(Debug)]
pub struct PeerStore {
    peers: HashMap<[u8; 20], Vec<PeerEntry>>,
}

impl PeerStore {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }

    /// Store a peer for the given info_hash
    pub fn add_peer(&mut self, info_hash: &[u8; 20], url: Url) {
        let entries = self.peers.entry(*info_hash).or_default();

        // avoid duplicates (same host)
        if entries.iter().any(|e| e.url.host == url.host) {
            // update the timestamp
            if let Some(existing) = entries.iter_mut().find(|e| e.url.host == url.host) {
                existing.added = time::Instant::now();
            }
            return;
        }

        entries.push(PeerEntry {
            url,
            added: time::Instant::now(),
        });
    }

    /// Get all non-expired peers for the given info_hash.
    /// Returns None if no peers are known.
    pub fn get_peers(&self, info_hash: &[u8; 20]) -> Option<Vec<&Url>> {
        let now = time::Instant::now();
        let ttl = time::Duration::from_secs(PEER_TTL_SECS);

        self.peers.get(info_hash).and_then(|entries| {
            let active: Vec<&Url> = entries
                .iter()
                .filter(|e| now.duration_since(e.added) < ttl)
                .map(|e| &e.url)
                .collect();
            if active.is_empty() {
                None
            } else {
                Some(active)
            }
        })
    }

    /// Remove expired entries across all info_hashes
    pub fn evict_expired(&mut self) {
        let now = time::Instant::now();
        let ttl = time::Duration::from_secs(PEER_TTL_SECS);

        self.peers.retain(|_, entries| {
            entries.retain(|e| now.duration_since(e.added) < ttl);
            !entries.is_empty()
        });
    }

    /// Number of info_hashes with at least one peer stored
    pub fn len(&self) -> usize {
        self.peers.len()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use uttd::url::{Scheme, Url};

    fn make_url(port: u16) -> Url {
        Url::from_ip_bytes(&[127, 0, 0, 1], port, Scheme::UDP)
    }

    #[test]
    fn add_and_get_peer() {
        let mut store = PeerStore::new();
        let hash = [0xAA; 20];
        store.add_peer(&hash, make_url(6881));
        store.add_peer(&hash, make_url(6882));

        let peers = store.get_peers(&hash).unwrap();
        assert_eq!(peers.len(), 2);
    }

    #[test]
    fn no_peers_returns_none() {
        let store = PeerStore::new();
        let hash = [0xBB; 20];
        assert!(store.get_peers(&hash).is_none());
    }

    #[test]
    fn duplicate_peer_not_added() {
        let mut store = PeerStore::new();
        let hash = [0xCC; 20];
        store.add_peer(&hash, make_url(6881));
        store.add_peer(&hash, make_url(6881));

        let peers = store.get_peers(&hash).unwrap();
        assert_eq!(peers.len(), 1);
    }

    #[test]
    fn different_hashes_separate() {
        let mut store = PeerStore::new();
        let hash1 = [0xAA; 20];
        let hash2 = [0xBB; 20];

        store.add_peer(&hash1, make_url(6881));
        store.add_peer(&hash2, make_url(6882));

        assert_eq!(store.len(), 2);
        assert_eq!(store.get_peers(&hash1).unwrap().len(), 1);
        assert_eq!(store.get_peers(&hash2).unwrap().len(), 1);
    }
}
