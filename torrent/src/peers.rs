use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

use uttd::{url::Url, utp::UtpPacket, AsyncStream, AsyncStreamType, UttdError};

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
            let handle = tokio::spawn(Self::initiate_handshake(url, bytes));
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

    async fn initiate_handshake(
        url: Url,
        handshake_bytes: Arc<Vec<u8>>,
    ) -> Result<AsyncStream, UttdError> {
        tokio::select! {
            res = Self::initiate_handshake_tcp(&url, handshake_bytes.clone()) => {
                res
            }

            res = Self::handshake_utp(&url) => {
                res
            }


        }
    }

    async fn initiate_handshake_tcp(
        url: &Url,
        handshake_bytes: Arc<Vec<u8>>,
    ) -> Result<AsyncStream, UttdError> {
        let mut stream = tokio::time::timeout(
            Duration::from_secs(5),
            tokio::net::TcpStream::connect(&url.host),
        )
        .await??;

        let mut res = vec![0; 68];
        // let result = stream?.write_all(&handshake_bytes, &mut res);
        let br = AsyncStream::send_tcp(&mut stream, &handshake_bytes, &mut res).await?;

        // Though the docs proclaim that "all current implmentation" of the bittorrent protocol
        // set all the reserved bytes to 0, most peer support atleast a few extentions, most
        // torrent clients also modify some reserved bytes. `reserved[5]` and `reserved[7]` is usually set.
        // `reserved[5]` == 0x10 indicates that this peer actually supports the extended bittorrent protocol,
        // while reserved[7] == 0x04 indicates the peer suppor the fast extention. `reserved[7]` == 0x01 indicates
        // that the peer supports DHT nodes. Combining both, `reserved[7]` == 0x05 means that the peer supports
        // both the fast extention and the DHT extention.
        //
        // As such, the response reserved bytes from a received packet generally looks like this:
        //  0  1  2  3  4  5   6  7
        // [0, 0, 0, 0, 0, 16, 0, 5]

        // IMPORTANT: this checks if the peer supports the extended bittorret protocol
        // https://www.bittorrent.org/beps/bep_0010.html
        // Peers announce to each other whether or not they support the extended protocol by setting the
        // reserved[5](0-indexed) byte to 0x10 (16 in decimal)

        // if res[25] & 0x10 != 0 {
        //     println!("Found Extended",);
        // }
        //

        // [0, 0, 0, 197]
        // [20, 0, 100, 49, 50, 58, 99, 111, 109, 112, 108, 101, 116, 101, 95, 97, 103, 111, 105, 50, 54, 49, 56, 101, 49, 58, 109, 100]
        //  |
        //  |
        //  |
        //  |
        //  This is the extended byte message.
        // TODO: figure out the message
        // https://www.bittorrent.org/beps/bep_0010.html

        // TODO: restruct these elsewhere
        let mut dht_msg_len = vec![0; 4];

        AsyncStream::read_multiple_tcp(&mut stream, &mut dht_msg_len).await?;;
        let dht_msg_len = u32::from_be_bytes(dht_msg_len[0..4].try_into().unwrap());

        // TODO: make this beautiful
        let mut temp = vec![0; 2];
        AsyncStream::read_multiple_tcp(&mut stream, &mut temp).await?;;

        if temp[0] == 20 {
            let mut dht_msg = vec![0_u8; dht_msg_len as usize - 2];
            AsyncStream::read_multiple_tcp(&mut stream, &mut dht_msg).await?;;
            _ = bencode::bencode::decode(&mut dht_msg.into_iter()).expect("Can't decode bencode");
        }

        // `dht_msg` is u8-bytes of bencoded dictionary with various keys
        // see more: https://www.bittorrent.org/beps/bep_0010.html

        // [0, 0, 0, 197]
        // [20, 0, 100, 49, 50, 58, 99, 111, 109, 112, 108, 101, 116, 101, 95, 97, 103, 111, 105, 50, 54, 49, 56, 101, 49, 58, 109, 100]
        // println!(
        //     "Initial Value len: {}",
        //     u32::from_be_bytes([dht_msg[0], dht_msg[1], dht_msg[2], dht_msg[3]])
        // );
        //
        //
        // -----------------------------------------------------------------------------------------------------------------------------------
        //
        //

        if br == 68 && res[0] == 19 {
            Ok(AsyncStream {
                async_stream_type: AsyncStreamType::TcpStream(stream),
            })
        } else {
            Err(UttdError::FailedRequest)
        }
    }

    async fn handshake_utp(url: &Url) -> Result<AsyncStream, UttdError> {
        let mut stream = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
        stream.connect(&url.host).await?;

        let bytes = UtpPacket::new().as_bytes();
        let mut res = vec![0; 20];

        let _ = AsyncStream::send_utp(&mut stream, &bytes, &mut res).await?;

        // println!("Gets here with: br: {} & res: {}", br, res[0]);
        // println!("{:?}", url);

        if res[0] == 33 {
            return Ok(AsyncStream {
                async_stream_type: AsyncStreamType::UtpStream(stream),
            });
        }

        // TODO: remove this line
        Err(UttdError::FailedRequest)
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
        assert!(!streams.is_empty());
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
        assert!(!streams.is_empty());
    }
}
