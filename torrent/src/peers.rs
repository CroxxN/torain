#![allow(dead_code)]

use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct Peers {
    interval: u32,
    seeders: u32,
    leechers: u32,
    peer: Vec<Ipv4Addr>,
}

impl Peers {
    fn new(interval: u32, seeders: u32, leechers: u32, ip: Ipv4Addr) -> Self {
        Self {
            interval,
            seeders,
            leechers,
            peer: vec![ip],
        }
    }
    fn add_ip(&mut self, ip: Ipv4Addr) {
        self.peer.push(ip)
    }
}
