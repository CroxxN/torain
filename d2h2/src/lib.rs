// REFERENCE: https://www.bittorrent.org/beps/bep_0005.html

// TODO: make query response logic a seperate module
//
#![allow(dead_code)]

mod error;
mod request;

use uttd::{url::Url, AsyncStream};

#[derive(Debug)]
pub struct DHT {
    node: Node,
    table: RTable,
}

impl DHT {
    pub async fn new() -> Self {
        todo!()
    }
}

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
