#![allow(dead_code)]

use crate::torrent::{FileMode, Torrent};
use std::collections::HashMap;

pub struct TrackerParams<'a> {
    pub info_hash: &'a [u8],
    pub peer_id: [u8; 20],
    pub port: [u8; 4],
    pub uploaded: &'a [u8],
    pub downloaded: &'a [u8],
    pub left: Vec<u8>,
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
        let port = [b'6', b'8', b'8', b'1'];
        let left = format!("{}", Self::calculate_left(&torrent))
            .as_bytes()
            .to_vec();
        Self {
            info_hash: torrent.hash.as_slice(),
            peer_id,
            port,
            uploaded: &[b'0'],
            downloaded: &[b'0'],
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

    fn params(&self) -> HashMap<&'static str, &[u8]> {
        let mut map = HashMap::new();
        map.insert("info_hash", self.info_hash);
        map.insert("peer_id", self.peer_id.as_slice());
        map.insert("port", self.port.as_slice());
        map.insert("uploaded", self.uploaded);
        map.insert("downloaded", self.downloaded);
        map.insert("left", &self.left);
        map.insert("compact", self.compact);

        let event: &str = self.event.into();
        map.insert("event", event.as_bytes());
        map
    }
}

#[cfg(test)]
mod test {
    use uttd::urutil::build_url;
    use uttd::Stream;

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
    fn announce() {
        let fs = "debian.torrent";
        let torrent = Torrent::from_file(fs).unwrap();
        let tracker = TrackerParams::new(&torrent);
        let params = &tracker.params();
        let url = &torrent.announce;
        let mut stream = Stream::new(url).unwrap();
        let path = build_url(&url.location, params);
        let res = stream.get(&path).unwrap();

        assert_eq!([res[9], res[10], res[11]], [b'2', b'0', b'0']);
    }
}
