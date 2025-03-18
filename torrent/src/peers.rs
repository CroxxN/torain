use std::sync::Arc;
use tokio::sync::Mutex;

use uttd::{url::Url, AsyncStream};

#[derive(Debug)]
pub struct Peers {
    pub interval: i32,
    pub seeders: i32,
    pub leechers: i32,
    pub peer: Vec<Url>,
}

// async fn handshakes(url: Url, handshake: Arc<Vec<u8>>) -> Result<AsyncStream, UttdError> {
//     let mut stream = AsyncStream::new(&url).await;
//     if let Ok(ast) = &mut stream {
//         let mut res = vec![0; 68];
//         if let Ok(bytes_read) = ast.send(&handshake, &mut res).await {
//             println!("{:?}", res);
//             if bytes_read == 68 && res[0] == 19 {
//                 return stream;
//             };
//         };
//     };
//     Err(UttdError::FailedRequest)
// }

// it works... type shit
// async fn handshakes(url: Url, handshake: Arc<Vec<u8>>) -> Result<AsyncStream, UttdError> {
//     // let stream = AsyncStream::new(&url).await;
//     // if let Err(e) = stream {
//     //     return Err(e);
//     // }
//     // let mut stream = stream.unwrap();
//     let mut utp_stream = UtpStream::new(&url).await?;
//     let utp_data = UtpPacket::new().as_bytes();

//     let _res = vec![0; 68];
//     let mut utp_res = vec![0; 20];

//     // utp_stream.send(&utp_data, &mut utp_res).await;
//     utp_stream.send(&utp_data, &mut utp_res).await;

//     // TODO: fix this. Maybe use the new function directly
//     let stream = AsyncStream::handshake(&url, handshake).await?;

//     if utp_res[0] == 33 {
//         return Ok(stream);
//     }
//     // }
//     Err(UttdError::FailedRequest)
// }

// #[tokio::test]
// async fn test_handshake() {
//     let url = Url::from_ip("39.42.236.140", 42245).unwrap();
//     let handshake: Arc<Vec<u8>> = Arc::new(Vec::new());

//     println!("{:?}", handshakes(url, handshake).await);
// }

impl Peers {
    pub fn new(interval: i32, seeders: i32, leechers: i32, ip: Vec<Url>) -> Self {
        Self {
            interval,
            seeders,
            leechers,
            peer: ip,
        }
    }
    pub async fn handshake(
        self,
        info_hash: [u8; 20],
        peer_id: [u8; 20],
    ) -> Vec<Arc<Mutex<AsyncStream>>> {
        // let peer: Vec<Arc<Url>> = self.peer.clone().into_iter().map(|x| Arc::new(x)).collect();
        let peer = self.peer;
        let mut handshake = Handshake::new(info_hash, peer_id);
        let handshake_bytes: Arc<Vec<u8>> = Arc::new(handshake.as_bytes_mut().to_vec());

        let mut handles = Vec::with_capacity(peer.len());

        let mut successful_streams = Vec::with_capacity(peer.len());

        for url in peer {
            let bytes = handshake_bytes.clone();
            let handle = tokio::spawn(uttd::AsyncStream::new(url, bytes));
            handles.push(handle);
        }

        for handle in handles {
            let res = handle.await.unwrap();
            if let Ok(r) = res {
                let r = Arc::new(Mutex::new(r));
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
        // Setting the last bit of the reserved field to indicate we support DHT
        let mut reserved = [0u8; 8];
        // TODO: fix
        reserved[7] |= 0x01;
        reserved[5] |= 0x10;
        Self {
            len: 19,
            protocol: *b"BitTorrent protocol",
            reserved,
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

    // // WARNING: This may fail
    #[test]
    fn connect_test() {
        let peer = "112.156.141.234:4681";

        let fs = "pulpfiction.torrent";
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
        // data.extend_from_slice(&[0; 7]);
        // data.extend_from_slice(&[0x01]);
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

    // IMPORTANT: This may fail as there usually aren't many healthy peers on HTTP/UDP based trackers
    #[tokio::test]
    async fn utp() {
        let fs = "pulpfiction.torrent";
        let torrent = Torrent::from_file(fs).unwrap();
        let tracker = TrackerParams::new(&torrent);
        let announce = tracker.announce().unwrap();
        let info_hash = torrent.hash;
        let peer_id = tracker.peer_id;
        let streams = announce.handshake(info_hash, peer_id).await;
        assert!(streams.len() != 0);
    }
}
