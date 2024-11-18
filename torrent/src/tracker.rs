use std::collections::HashMap;
use uttd::urutil::build_url;

pub struct TrackerParams {
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
    pub port: [u8; 4],
    pub uploaded: u8,
    pub downloaded: u8,
    pub left: usize,
    pub compact: u8,
    pub event: Event,
    pub trackerid: Option<[u8; 20]>,
}

pub enum Event {
    Started,
    Stopped,
    Completed,
}

impl TrackerParams {
    fn new(info_hash: [u8; 20], port: u32) -> Self {
        let pid = std::process::id();
        let peer_id = format!("TORAIN-CROXX--{}", pid).into_bytes()[..20]
            .try_into()
            .unwrap();
        let port = format!("{}", port).as_bytes().try_into().unwrap();
        Self {
            info_hash,
            peer_id,
            port,
            uploaded: 0,
            downloaded: 0,
            left: 0,
            compact: 0,
            event: Event::Started,
            trackerid: None,
        }
    }
    fn hash(self) -> HashMap<&'static str, Vec<u8>> {
        let mut map = HashMap::new();
        map.insert("info_hash", self.info_hash.into());
        map.insert("peer_id", self.peer_id.into());
        map.insert("port", self.port.into());
        map.insert("uploaded", vec![b'0']);
        map.insert("downloaded", vec![b'0']);
        map.insert("left", vec![b'0']);
        map.insert("compact", vec![b'0']);
        map.insert("event", "started".to_owned().into_bytes());
        map
    }

    pub fn request(base: &str, map: HashMap<&str, Vec<u8>>) {
        let url = build_url(base, map);
        let request_header = format!(
            "GET {} HTTP/1.1\r\n
            Host: {}\r\n
            Connection: close\r\n
            User-agent: torain\r\n
            Accept: */*\r\n
            ",
            url
        );
    }
}

#[cfg(test)]
mod test {
    use super::TrackerParams;

    #[test]
    fn test_peerid() {
        let params = TrackerParams::new([0; 20], 6118);
        assert_eq!(params.peer_id.len(), 20);
    }

    #[test]
    fn url() {
        let params = TrackerParams::new([0; 20], 6118);
        let map = params.hash();
        assert_eq!(TrackerParams::request("https", map), ());
    }
}
