// REFERENCE: https://www.bittorrent.org/beps/bep_0005.html

// TODO:
// = implement a routing table
// = implement the XOR metric
// = more

#![allow(dead_code)]

mod error;
mod request;

use crypto::sha1;
use error::DHTError;
use std::time;
use uttd::{url::Url, AsyncStream};

#[derive(Debug)]
pub struct Node {
    id: [u8; 20],
    routing: RTable,
}

// TODO: implement node
impl Node {}

// Routing table
#[derive(Debug)]
pub struct RTable {}

// TODO: Implement the routing table
impl RTable {
    pub fn new() -> Self {
        Self {}
    }
}

// we need a few nodes to "bootstrap" our DHT table.
// pub async fn bootstrap() -> Result<(), UttdError> {
// List of other bootstrap nodes:
//
// "router.bittorrent.com:6881",
// "router.utorrent.com:6881",
// "router.bitcomet.com:6881",
// "dht.transmissionbt.com:6881",
// "dht.aelitis.com:6881",

// TODO: Complete
pub async fn bootstrap() -> AsyncStream {
    // we use "udp" here, despite not requiring the scheme, to let
    // our url library know that we want to create a *UDP* async stream.
    let bootstrap = Url::new("udp://router.bittorrent.com:6881").unwrap();

    AsyncStream::new(bootstrap).await.unwrap()
}

#[derive(Debug)]
pub struct DHT {
    node_id: [u8; 20],
    table: RTable,
}

impl DHT {
    fn new() -> Result<Self, DHTError> {
        // Generate node_id using two phases
        // BEP_0005 recommends entory for node id generation which is
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
            table: RTable::new(),
        })
    }
}
