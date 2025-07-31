#![allow(dead_code)]

use crate::error::DHTError;
use crypto::sha1;
use std::time;

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

#[derive(Debug)]
pub struct DHT {
    node_id: [u8; 20],
    table: RouteTable,
}

impl DHT {
    fn new() -> Result<Self, DHTError> {
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

// Routing table
#[derive(Debug)]
pub struct RouteTable {}

// TODO: Implement the routing table
impl RouteTable {
    pub fn new() -> Self {
        // PLACEHOLDER:
        Self {}
    }
}
