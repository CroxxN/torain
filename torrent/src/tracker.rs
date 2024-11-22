#![allow(dead_code)]

use ::bencode::bencode::BTypes;
use ::bencode::utils::BencodeErr;
use bencode::bencode;
use uttd::urutil::{build_url, parse_response};
use uttd::StreamType;
use uttd::{url::Url, Stream};

use crate::torrent::{FileMode, Torrent};
use std::collections::{BTreeMap, HashMap};

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
    fn new(torrent: &'a Torrent) -> Self {
        let peer_id = "--sd--TORAIN---01523".as_bytes()[..20].try_into().unwrap();
        let port = 6881;
        let left = Self::calculate_left(&torrent) as u64;
        Self {
            url: torrent.announce.clone(),
            info_hash: torrent.hash.as_slice(),
            peer_id,
            port,
            uploaded: 0,
            downloaded: 0,
            left,
            compact: &[b'0'],
            event: Event::Started,
            trackerid: None,
        }
    }

    // will be more sophisticated once resume function is implemented
    fn calculate_left(torrent: &Torrent) -> usize {
        match torrent.info.mode {
            FileMode::SingleMode { length } => length,
            FileMode::MultiMode { ref files } => {
                files.iter().map(|f| f.length).fold(0, |acc, l| acc + l)
            }
        }
    }

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
    pub fn announce(&self) -> Result<BTreeMap<String, BTypes>, BencodeErr> {
        match self.url.scheme {
            uttd::url::Scheme::UDP => self.announce_udp(),
            _ => self.announce_tcp(),
        }
    }

    fn announce_tcp(&self) -> Result<BTreeMap<String, BTypes>, BencodeErr> {
        let params = &self.params();
        let url = &self.url;
        let path = build_url(&url.location, params);
        let mut stream = Stream::new(url).unwrap();
        let mut res = stream.get(&path).unwrap();
        let mut body = parse_response(uttd::url::Scheme::HTTP, &mut res)
            .unwrap()
            .to_vec()
            .into_iter();
        let decoded_body = bencode::decode(&mut body).unwrap();
        if let BTypes::DICT(d) = decoded_body {
            return Ok(d);
        } else {
            Err(BencodeErr::Berr)
        }
        // assert_eq!([res[9], res[10], res[11]], [b'2', b'0', b'0']);
    }
    fn announce_udp(&self) -> Result<BTreeMap<String, BTypes>, BencodeErr> {
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

        // let mut body = parse_response(uttd::url::Scheme::UDP, &mut res)
        //     .unwrap()
        //     .to_vec()
        //     .into_iter();
        // let decoded_body = bencode::decode(&mut body).unwrap();
        // if let BTypes::DICT(d) = decoded_body {
        //     return Ok(d);
        // } else {
        //     Err(BencodeErr::Berr)
        // }

        Ok(BTreeMap::new())
    }
}

#[cfg(test)]
mod test {

    use std::collections::BTreeMap;

    use super::TrackerParams;
    use crate::torrent::Torrent;

    #[test]
    fn left_single_mode() {
        let fs = "debian.torrent";
        let torrent = Torrent::from_file(fs).unwrap();
        if let crate::torrent::FileMode::SingleMode { length } = torrent.info.mode {
            let left = TrackerParams::calculate_left(&torrent);
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
            let left = TrackerParams::calculate_left(&torrent);
            assert_eq!(left, length);
        }
    }

    #[test]
    fn announce_tcp() {
        let fs = "debian.torrent";
        let torrent = Torrent::from_file(fs).unwrap();
        let tracker = TrackerParams::new(&torrent);
        assert!(!tracker.announce().unwrap().is_empty());
    }

    #[test]
    fn announce_udp() {
        let fs = "pulpfiction.torrent";
        let torrent = Torrent::from_file(fs).unwrap();
        let tracker = TrackerParams::new(&torrent);
        assert_eq!(tracker.announce().unwrap(), BTreeMap::new());
    }
}
