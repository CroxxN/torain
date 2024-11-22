#![allow(dead_code)]

use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct Peers {
    interval: u32,
    peer: Vec<Peer>,
}

#[derive(Debug)]
struct Peer {
    ip: Ipv4Addr,
    port: u16,
}

impl Peers {
    fn new(raw: &[u8]) {
        todo!()
    }
}
