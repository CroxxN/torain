use std::{path::PathBuf, str::FromStr, sync::Arc};
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;

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
    BitField(Bitfield),
    // BitField(Vec<u8>),
    Piece(Vec<u8>),
    Finish,
}

#[derive(Debug)]
pub enum DownloadError {
    Unrecognized,
}

impl Message {
    fn try_from(value: &[u8], len: usize) -> Self {
        let message = match value[0] {
            // id
            0 => Self::Choke,
            1 => Self::Unchoke,
            2 => Self::Interested,
            3 => Self::NotInterested,
            4 => Self::Have(u32::from_be_bytes(value[1..5].try_into().unwrap())),
            5 => Self::BitField(Bitfield::from_bytes(&value[1..(len - 1)], value.len())),
            7 => Self::Piece(value[1..(len - 9)].to_vec()),
            13 => Self::Request,
            _ => todo!(),
        };
        message
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bitfield {
    bit: BitfieldInner,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BitfieldInner {
    Large(Vec<u8>),
    Compact(u64),
}

impl Bitfield {
    fn new(cap: usize) -> Self {
        Self {
            bit: BitfieldInner::Large(vec![0; (cap / 8) + 1]),
        }
    }

    pub fn new_compact() -> Self {
        Self {
            bit: BitfieldInner::Compact(0),
        }
    }

    pub fn len(&self) -> usize {
        match &self.bit {
            BitfieldInner::Large(x_compact) => x_compact.len(),
            BitfieldInner::Compact(_) => 64,
        }
    }

    pub fn from_bytes(bits: &[u8], cap: usize) -> Self {
        let mut bfield = Self::new(8 * (cap - 1));
        if let BitfieldInner::Large(x_field) = &mut bfield.bit {
            bits.iter().enumerate().for_each(|(i, x)| {
                let x = u8::from_be(*x);
                x_field[i] = x;
            });
        }
        bfield
    }

    pub fn set(&mut self, index: usize) {
        match &mut self.bit {
            BitfieldInner::Large(x_field) => {
                let max_point = (index / 8) as usize;
                let min_point = index % 8;
                x_field[max_point] = x_field[max_point] ^ (1 << min_point);
            }
            BitfieldInner::Compact(x_compac) => {
                self.bit = BitfieldInner::Compact(*x_compac ^ (1 << index));
            }
        }
    }

    pub fn get(&mut self, index: u64) -> u64 {
        match &mut self.bit {
            BitfieldInner::Large(x_field) => {
                let max_point = (index / 8) as usize;
                let min_point = index % 8;
                let res = x_field[max_point] >> min_point;
                res as u64
            }
            BitfieldInner::Compact(x_compac) => *x_compac >> index,
        }
    }
}

#[test]
fn bitfield_large() {
    let mut bits = Bitfield::new(10);
    bits.set(4);
    assert_eq!(bits.get(4), 1);
    assert!(bits.len() == 2);
}

#[test]
fn bitfield_compact() {
    let mut bits = Bitfield::new_compact();
    bits.set(4);
    assert_eq!(bits.get(4), 1);
    assert!(bits.len() == 64);
}

#[derive(Debug)]
pub struct Participants {
    pub file_size: usize,
    pub block_size: usize,
    pub peers: Vec<Arc<Mutex<AsyncStream>>>,
    pub path: PathBuf,
}

impl Participants {
    pub async fn new(t: &Torrent, peers: Vec<Arc<Mutex<AsyncStream>>>) -> Self {
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

    // continous reading
    async fn listen_peers(url: Arc<Mutex<AsyncStream>>, tx: Sender<(usize, Message)>, idx: usize) {
        loop {
            let message_len = url.lock().await.read_once().await.unwrap();
            if message_len == 0 {
                return;
                // return Message::KeepAlive;
            }
            let mut message = vec![0u8; message_len as usize];
            url.lock().await.read_multiple(&mut message).await.unwrap();
            let which_message: Message =
                Message::try_from(message.as_slice(), message_len as usize);
            tx.send((idx, which_message)).await.unwrap();
        }
    }
    pub async fn download(self) {
        // TODO: Remove unwrap

        // _ = tokio::fs::create_dir("./.torain_temp").await;
        let peers_amt = self.peers.len();
        assert!(peers_amt > 0);

        // am i choking the remote peer?
        let mut _am_chok: Bitfield = Bitfield::new_compact();
        // am i interested in the remote peer?
        let mut _am_interested = Bitfield::new_compact();

        // is the remote peer choking me?
        let mut peer_chok = Bitfield::new_compact();
        // is the remote peer interested in me?
        let mut peer_interested = Bitfield::new_compact();

        let mut piece_map: Vec<Bitfield> = Vec::new();

        let (tx, mut rx) = mpsc::channel::<(usize, Message)>(peers_amt);
        let mut handles = vec![];

        for (idx, url) in self.peers.into_iter().enumerate() {
            let txc = tx.clone();
            let hnd = tokio::spawn(async move { Self::listen_peers(url, txc, idx).await });
            handles.push(hnd);
        }

        // continous receiving from multiple sources
        loop {
            if let Some((idx, value)) = rx.recv().await {
                // TODO:
                match value {
                    Message::BitField(x_field) => piece_map.push(x_field),
                    Message::Choke => peer_chok.set(idx),
                    Message::Unchoke => peer_chok.set(idx),
                    Message::Interested => peer_interested.set(idx),
                    _ => {
                        println!("{:?}", value);
                        unimplemented!()
                    }
                };
            }
            println!("{:?}", piece_map);
        }
    }
}

#[cfg(test)]
mod test {
    // use crate::download::Participants;
    // use crate::{torrent::Torrent, tracker::TrackerParams};

    // #[tokio::test]
    // async fn listen() {
    //     let fs = "debian.torrent";
    //     // let fs = "pulpfiction.torrent";
    //     // let fs = "nocountry.torrent";
    //     let torrent = Torrent::from_file(fs).unwrap();
    //     let tracker = TrackerParams::new(&torrent);
    //     let announce = tracker.announce().unwrap();
    //     let info_hash = torrent.hash;
    //     let peer_id = tracker.peer_id;
    //     let streams = announce.handshake(info_hash, peer_id).await;
    //     assert!(streams.len() > 1);
    //     let down = Participants::new(&torrent, streams).await;
    //     down.download().await;
    // }
}
