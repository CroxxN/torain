use std::{path::PathBuf, str::FromStr};

use uttd::AsyncStream;

use crate::torrent::Torrent;

pub enum Message {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32),
    Unrecognized,
    Request,
}

#[derive(Debug)]
pub enum DownloadError {
    Unrecognized,
}

impl TryFrom<&[u8]> for Message {
    type Error = DownloadError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let message = match value[0] {
            0 => Self::Choke,
            1 => Self::Unchoke,
            2 => Self::Interested,
            3 => Self::NotInterested,
            4 => Self::Have(u32::from_be_bytes(value[6..10].try_into().unwrap())),
            13 => Self::Request,
            _ => todo!(),
        };
        Ok(message)
    }
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
    pub async fn start_download(self) {
        // TODO: Remove unwrap
        tokio::fs::create_dir("./.torain_temp").await.unwrap();
        for _url in self.peers {
            loop {
                todo!()
            }
        }
    }
}

async fn _listen_peers(mut url: AsyncStream) {
    let message_len = url.read_once().await.unwrap();
    if message_len == 0 {
        return;
    }

    let mut message = vec![0u8; message_len as usize];
    url.read_multiple(&mut message).await.unwrap();
    let which_message: Message = message.as_slice().try_into().unwrap();

    match which_message {
        Message::Choke => todo!(),
        _ => todo!(),
    }
}

mod test {
    #[tokio::test]
    async fn test() {}
}
// pub async fn download(peers: ){

// }
