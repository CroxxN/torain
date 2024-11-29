#![allow(dead_code)]

use ::bencode::bencode::BTypes;
use ::bencode::utils::BencodeErr;
use bencode::bencode;
use uttd::urutil::{build_url, response};
use uttd::StreamType;
use uttd::{url::Url, Stream};

use crate::peers::Peers;
use crate::torrent::Torrent;
use core::panic;
use std::collections::HashMap;

pub struct TrackerParams<'a> {
    pub url: Url,
    pub info_hash: &'a [u8],
    pub peer_id: [u8; 20],
    pub port: u16,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
    pub compact: &'a [u8],
    pub event: Event,
    pub trackerid: Option<[u8; 20]>,
}

#[derive(Clone, Copy)]
pub enum Event {
    Started,
    Stopped,
    Completed,
}

impl From<Event> for &str {
    fn from(value: Event) -> Self {
        match value {
            Event::Started => "started",
            Event::Stopped => "stopped",
            Event::Completed => "completed",
        }
    }
}

impl<'a> TrackerParams<'a> {
    pub fn new(torrent: &'a Torrent) -> Self {
        let peer_id = "--sd--TORAIN---01523".as_bytes()[..20].try_into().unwrap();
        let port = 6881;
        let left = torrent.calculate_left() as u64;
        Self {
            url: torrent.announce.clone(),
            info_hash: torrent.hash.as_slice(),
            peer_id,
            port,
            uploaded: 0,
            downloaded: 0,
            left,
            compact: &[b'1'],
            event: Event::Started,
            trackerid: None,
        }
    }

    // will be more sophisticated once resume function is implemented

    fn params(&self) -> HashMap<&'static str, Vec<u8>> {
        let mut map = HashMap::new();
        let left = format!("{}", self.left).as_bytes().to_vec();
        let uploaded = format!("{}", self.uploaded).as_bytes().to_vec();
        let downloaded = format!("{}", self.downloaded).as_bytes().to_vec();

        map.insert("info_hash", self.info_hash.to_vec());
        map.insert("peer_id", self.peer_id.to_vec());
        map.insert("port", self.port.to_be_bytes().to_vec());
        map.insert("uploaded", uploaded);
        map.insert("downloaded", downloaded);
        map.insert("left", left);
        map.insert("compact", self.compact.to_vec());

        let event: &str = self.event.into();
        map.insert("event", event.as_bytes().to_vec());
        map
    }
    pub fn announce(&self) -> Result<Peers, BencodeErr> {
        match self.url.scheme {
            uttd::url::Scheme::UDP => self.announce_udp(),
            _ => self.announce_tcp(),
        }
    }

    fn announce_tcp(&self) -> Result<Peers, BencodeErr> {
        let params = &self.params();
        let url = &self.url;
        let path = build_url(&url.location, params);
        let mut stream = Stream::new(url).unwrap();
        let mut res = stream.get(&path).unwrap();
        let response = response(uttd::url::Scheme::HTTP, &mut res).unwrap();
        let body = response.1.to_vec();
        let (interval, sock) = Self::bencoded_ip_mode(body);

        let peers = Peers::new(interval as i32, 0, 0, sock);

        Ok(peers)
        // response.interval = interval;
    }
    fn announce_udp(&self) -> Result<Peers, BencodeErr> {
        let mut request_body: Vec<u8> = Vec::new();

        let url = &self.url;
        let mut stream = Stream::new(url).unwrap();
        let connection_id;
        if let StreamType::UDP(ref u) = stream.stream {
            connection_id = u.connection_id;
        } else {
            return Err(BencodeErr::Berr);
        };

        request_body.extend_from_slice(&connection_id.to_be_bytes()); // connection_id
        request_body.extend_from_slice(&1_u32.to_be_bytes()); // action
        request_body.extend_from_slice(&1_u32.to_be_bytes()); // transaction_id
        request_body.extend_from_slice(&self.info_hash); // info_hash
        request_body.extend_from_slice(&self.peer_id); // peer_id
        request_body.extend_from_slice(&0_u64.to_be_bytes()); // downloaded
        request_body.extend_from_slice(&self.left.to_be_bytes()); // left
        request_body.extend_from_slice(&self.uploaded.to_be_bytes()); // uploaded
        request_body.extend_from_slice(&2_u32.to_be_bytes()); // event
        request_body.extend_from_slice(&0_u32.to_be_bytes()); // ip address
        request_body.extend_from_slice(&1_u32.to_be_bytes()); // key
        request_body.extend_from_slice(&(-1_i32).to_be_bytes()); // num_want
        request_body.extend_from_slice(&self.port.to_be_bytes()); // port

        assert!(request_body.len() == 98);

        let mut res = vec![0; 1024];
        stream.send(&request_body, &mut res).unwrap();

        let (info, body) = response(uttd::url::Scheme::UDP, &mut res).unwrap();
        let body = body.to_vec();
        let res = Self::compact_ip_mode(body.as_slice());

        let peers = Peers::new(info.interval, info.seeders, info.leechers, res);

        Ok(peers)
    }

    pub fn compact_ip_mode(bytes: &[u8]) -> Vec<Url> {
        let ips: Vec<Url> = bytes
            .chunks(6)
            .map(|x| {
                let ip: [u8; 4] = x[0..4].try_into().unwrap();
                let port = u16::from_be_bytes(x[4..6].try_into().unwrap());
                Url::from_ip_bytes(&ip, port)
            })
            .collect();

        ips
    }

    fn bencoded_ip_mode(bytes: Vec<u8>) -> (usize, Vec<Url>) {
        let mut ips = Vec::new();
        let mut interval = 0;

        let decoded_body = bencode::decode(&mut bytes.into_iter()).unwrap();
        if let BTypes::DICT(d) = decoded_body {
            if let Some(_) = d.get("failure") {
                panic!("FAILED");
            }
            interval = d.get("interval").unwrap().try_into().unwrap();
            let peers = d.get("peers").unwrap();
            if let BTypes::LIST(l) = peers {
                l.iter().for_each(|peers| {
                    if let BTypes::DICT(peer_list) = peers {
                        let ip: String = peer_list.get("ip").unwrap().try_into().unwrap();
                        let port: usize = peer_list.get("port").unwrap().try_into().unwrap();
                        ips.push(Url::from_ip(&ip, port as u16).unwrap());
                    };
                });
            } else if let BTypes::BSTRING(bpeers) = peers {
                return (interval, Self::compact_ip_mode(bpeers));
            }
        };
        (interval, ips)
    }
}

#[cfg(test)]
mod test {

    use uttd::url::Url;

    use super::TrackerParams;
    use crate::torrent::Torrent;

    #[test]
    fn left_single_mode() {
        let fs = "debian.torrent";
        let torrent = Torrent::from_file(fs).unwrap();
        if let crate::torrent::FileMode::SingleMode { length } = torrent.info.mode {
            let left = torrent.calculate_left();
            assert_eq!(left, length);
        }
    }

    #[test]
    fn left_multi_mode() {
        let fs = "pulpfiction.torrent";
        let torrent = Torrent::from_file(fs).unwrap();
        if let crate::torrent::FileMode::MultiMode { ref files } = torrent.info.mode {
            let mut length = 0;
            for f in files {
                length += f.length;
            }
            let left = torrent.calculate_left();
            assert_eq!(left, length);
        }
    }

    #[test]
    fn announce_tcp() {
        let fs = "debian.torrent";
        let torrent = Torrent::from_file(fs).unwrap();
        let tracker = TrackerParams::new(&torrent);
        let announce = tracker.announce().unwrap();
        assert!(!announce.peer.is_empty());
    }

    #[test]
    fn announce_udp() {
        let fs = "pulpfiction.torrent";
        let torrent = Torrent::from_file(fs).unwrap();
        let tracker = TrackerParams::new(&torrent);
        let announce = tracker.announce().unwrap();
        assert!(!announce.peer.is_empty());
    }

    #[test]
    fn parse_compact_ip() {
        let ip = &[127, 0, 0, 1, 31, 144, 0, 0, 0, 0, 0, 0];
        let mut expected = vec![Url::from_ip_bytes(&[127, 0, 0, 1], 8080)];
        expected.push(Url::from_ip_bytes(&[0, 0, 0, 0], 0));

        let ips = TrackerParams::compact_ip_mode(ip);
        assert_eq!(ips, expected);
    }

    #[test]
    fn parse_non_compact_ip() {
        let data = "d8:intervali100e5:peersld2:ip13:192.168.1.1054:porti6881eed2:ip9:127.0.0.14:porti8080eeee"
            .as_bytes()
            .to_vec();
        let res = TrackerParams::bencoded_ip_mode(data);
        let expected = vec![
            Url::from_ip("192.168.1.105", 6881).unwrap(),
            Url::from_ip("127.0.0.1", 8080).unwrap(),
        ];
        assert_eq!(res.1, expected);
    }
}
