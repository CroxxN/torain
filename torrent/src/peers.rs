#![allow(dead_code)]

use std::net::SocketAddr;

#[derive(Debug)]
pub struct Peers {
    pub interval: i32,
    pub seeders: i32,
    pub leechers: i32,
    pub peer: Vec<SocketAddr>,
}

impl Peers {
    pub fn new(interval: i32, seeders: i32, leechers: i32, ip: Vec<SocketAddr>) -> Self {
        Self {
            interval,
            seeders,
            leechers,
            peer: ip,
        }
    }
}
