use std::{path::PathBuf, str::FromStr};

use uttd::AsyncStream;

use crate::torrent::Torrent;

#[derive(Debug, Eq, PartialEq)]
pub enum Message {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32),
    Unrecognized,
    Request,
    // BitField(Bitfield),
    BitField(Vec<u8>),
}

#[derive(Debug)]
pub enum DownloadError {
    Unrecognized,
}

// impl TryFrom<&[u8]> for Message {
//     type Error = DownloadError;
// }

impl Message {
    //     fn as_bytes(&self) -> &[u8] {
    //         TODO: Lenght is 4 bytes big endian
    //         // match self {
    //         //     Self::Choke => &[0],
    //         //     Self::Unchoke => &[1],
    //         // }
    //     }
    fn try_from(value: &[u8], len: usize) -> Self {
        let message = match value[0] {
            0 => Self::Choke,
            1 => Self::Unchoke,
            2 => Self::Interested,
            3 => Self::NotInterested,
            4 => Self::Have(u32::from_be_bytes(value[6..10].try_into().unwrap())),
            13 => Self::Request,
            // 5 => Self::BitField(Bitfield::default()),
            5 => Self::BitField(value[6..len].to_vec()),
            _ => todo!(),
        };
        message
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bitfield {
    bit: Vec<u8>,
}

impl Bitfield {
    fn new(cap: usize) -> Self {
        Self {
            bit: vec![0; (cap / 8) + 1],
        }
    }
    pub fn from_bytes(bits: &[u8], cap: usize) -> Self {
        let mut bfield = Self::new(cap);
        bits.iter().enumerate().for_each(|(i, x)| {
            let x = u8::from_be(*x);
            bfield.bit[i] = x;
        });
        bfield
    }

    pub fn set(&mut self, index: u64) {
        let max_point = (index / 8) as usize;
        let min_point = index % 8;
        self.bit[max_point] = self.bit[max_point] ^ (1 << min_point);
        // self.bit = self.bit ^ (1 << index);
    }

    pub fn get(&mut self, index: u64) -> u64 {
        let max_point = (index / 8) as usize;
        let min_point = index % 8;
        let res = self.bit[max_point] >> min_point;
        res as u64
    }
}

#[test]
fn bitfield() {
    let mut bits = Bitfield::new(10);
    bits.set(4);
    assert_eq!(bits.get(4), 1);
    assert!(bits.bit.len() == 2);
}

#[derive(Debug)]
pub struct Participants {
    pub file_size: usize,
    pub block_size: usize,
    pub peers: Vec<AsyncStream>,
    pub path: PathBuf,
}

impl Participants {
    pub async fn new(t: &Torrent, peers: Vec<AsyncStream>) -> Self {
        let file_size = t.calculate_left();
        let block_size = t.info.piece_length;
        let folder = &t.info.name;
        let path = PathBuf::from_str(&folder).unwrap();

        Self {
            file_size,
            block_size,
            peers,
            path,
        }
    }
    pub async fn start_download(mut self) {
        // TODO: Remove unwrap
        tokio::fs::create_dir("./.torain_temp").await.unwrap();
        loop {
            for url in &mut self.peers {
                listen_peers(url).await;
                todo!()
            }
        }
    }
}

async fn listen_peers(url: &mut AsyncStream) -> Message {
    let message_len = url.read_once().await.unwrap();
    if message_len == 0 {
        return Message::KeepAlive;
    }

    let mut message = vec![0u8; message_len as usize];
    url.read_multiple(&mut message).await.unwrap();
    let which_message: Message = Message::try_from(message.as_slice(), message_len as usize);

    which_message
}

#[cfg(test)]
mod test {
    use crate::download::{listen_peers, Message};
    use crate::{torrent::Torrent, tracker::TrackerParams};

    #[tokio::test]
    async fn listen() {
        let fs = "debian.torrent";
        let torrent = Torrent::from_file(fs).unwrap();
        let tracker = TrackerParams::new(&torrent);
        let announce = tracker.announce().unwrap();
        let info_hash = torrent.hash;
        let peer_id = tracker.peer_id;
        let streams = &mut announce.handshake(info_hash, peer_id).await[0];
        let v = listen_peers(streams).await;
        assert_eq!(v, Message::KeepAlive);
    }
}
// pub async fn download(peers: ){

// }
