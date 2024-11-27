#![allow(dead_code)]

use uttd::{url::Url, Stream};

#[derive(Debug)]
pub struct Peers {
    pub interval: i32,
    pub seeders: i32,
    pub leechers: i32,
    pub peer: Vec<Url>,
}

impl Peers {
    pub fn new(interval: i32, seeders: i32, leechers: i32, ip: Vec<Url>) -> Self {
        Self {
            interval,
            seeders,
            leechers,
            peer: ip,
        }
    }
    pub fn handshake(&self, info_hash: &[u8], peer_id: &[u8]) -> Vec<u8> {
        let peer = &self.peer;
        let mut res = vec![0; 68];
        let mut data = Vec::new();
        data.push(19_u8);
        let protocol = b"BitTorrent protocol";
        data.extend_from_slice(protocol);
        data.extend_from_slice(&[0; 8]);
        data.extend_from_slice(&info_hash[0..20]);
        data.extend_from_slice(&peer_id[0..20]);
        let mut stream = Stream::new(&peer[0]).unwrap();
        stream.send(&data, &mut res).unwrap();
        res
    }
}

#[cfg(test)]
mod test {
    use std::{
        io::{Read, Write},
        net::TcpStream,
    };

    use crate::{torrent::Torrent, tracker::TrackerParams};

    #[test]
    fn connect_test() {
        let peer = "193.5.17.149:31337";
        let fs = "debian.torrent";
        let torrent = Torrent::from_file(fs).unwrap();
        let tracker = TrackerParams::new(&torrent);
        let _peers = tracker.announce().unwrap();
        let info_hash = torrent.hash;
        let peer_id = tracker.peer_id;
        let mut data = Vec::new();
        data.push(19);
        let protocol = b"BitTorrent protocol";
        data.extend_from_slice(protocol);
        data.extend_from_slice(&[0; 8]);
        data.extend_from_slice(&info_hash[0..20]);
        data.extend_from_slice(&peer_id[0..20]);

        println!("{:?}", data);
        let mut stream = TcpStream::connect(peer).unwrap();
        stream.write_all(&data).unwrap();
        let mut res = vec![0; 68];
        stream.read_exact(&mut res).unwrap();
        assert!(res[0] == 19);
    }

    #[test]
    fn handshake_test() {
        let fs = "debian.torrent";
        let torrent = Torrent::from_file(fs).unwrap();
        let tracker = TrackerParams::new(&torrent);
        let announce = tracker.announce().unwrap();
        let info_hash = torrent.hash;
        let peer_id = tracker.peer_id;
        let res = announce.handshake(&info_hash, &peer_id);
        assert_eq!(res[0], 19);
    }
}
