use std::sync::Arc;

use uttd::{url::Url, AsyncStream, UttdError};

#[derive(Debug)]
pub struct Peers {
    pub interval: i32,
    pub seeders: i32,
    pub leechers: i32,
    pub peer: Vec<Url>,
}

async fn handshakes(url: Arc<Url>, handshake: Arc<Vec<u8>>) -> Result<AsyncStream, UttdError> {
    let mut stream = AsyncStream::new(&url).await;
    if let Ok(ast) = &mut stream {
        let mut res = vec![0; 68];
        if let Ok(bytes_read) = ast.send(&handshake, &mut res).await {
            if bytes_read == 68 && res[0] == 19 {
                return stream;
            };
        };
    };
    Err(UttdError::FailedRequest)
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
    pub async fn handshake(&self, info_hash: [u8; 20], peer_id: [u8; 20]) -> Vec<AsyncStream> {
        let peer: Vec<Arc<Url>> = self.peer.clone().into_iter().map(|x| Arc::new(x)).collect();
        let mut handshake = Handshake::new(info_hash, peer_id);
        let handshake_bytes: Arc<Vec<u8>> = Arc::new(handshake.as_bytes_mut().to_vec());

        let mut handles = Vec::with_capacity(peer.len());

        let mut successful_streams = Vec::with_capacity(peer.len());

        for url in peer {
            let bytes = handshake_bytes.clone();
            let handle = tokio::spawn(handshakes(url, bytes));
            handles.push(handle);
        }

        for handle in handles {
            let res = handle.await.unwrap();
            if let Ok(r) = res {
                successful_streams.push(r);
            }
        }
        successful_streams
    }
}

#[repr(C)]
#[repr(packed)]
pub struct Handshake {
    pub len: u8,
    pub protocol: [u8; 19],
    pub reserved: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Self {
            len: 19,
            protocol: *b"BitTorrent protocol",
            reserved: [0; 8],
            info_hash,
            peer_id,
        }
    }
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        let bytes = self as *mut Self as *mut [u8; std::mem::size_of::<Self>()];
        let bytes: &mut [u8; std::mem::size_of::<Self>()] = unsafe { &mut *bytes };
        bytes
    }
}

#[cfg(test)]
mod test {
    use std::{
        io::{Read, Write},
        net::TcpStream,
    };

    use crate::{peers::Handshake, torrent::Torrent, tracker::TrackerParams};

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

        let mut stream = TcpStream::connect(peer).unwrap();
        stream.write_all(&data).unwrap();
        let mut res = vec![0; 68];
        stream.read_exact(&mut res).unwrap();
        assert!(res[0] == 19);
    }

    #[tokio::test]
    async fn handshake_test() {
        let fs = "debian.torrent";
        let torrent = Torrent::from_file(fs).unwrap();
        let tracker = TrackerParams::new(&torrent);
        let _announce = tracker.announce().unwrap();
        let info_hash = torrent.hash;
        let peer_id = tracker.peer_id;
        let mut handshake = Handshake::new(info_hash, peer_id);
        let handshake_bytes = handshake.as_bytes_mut();
        assert_eq!(handshake_bytes[0], 19);
    }

    #[tokio::test]
    async fn streams() {
        let fs = "debian.torrent";
        let torrent = Torrent::from_file(fs).unwrap();
        let tracker = TrackerParams::new(&torrent);
        let announce = tracker.announce().unwrap();
        let info_hash = torrent.hash;
        let peer_id = tracker.peer_id;
        let streams = announce.handshake(info_hash, peer_id).await;
        assert!(streams.len() != 0);
    }
}
