use core::panic;

use bencode;
use bencode::bencode::BTypes;

pub struct Torrent {
    pub announce: String,
    pub announce_list: Option<Vec<String>>,
    pub creation_date: Option<i32>,
    pub comment: Option<String>,
    pub created_by: Option<String>,
    pub encoding: Option<String>,
    pub info: Info,
}

pub struct Info {
    pub name: String,
    pub piece_lenght: usize,
    pub pieces: Vec<u8>,
    pub mode: FileMode,
}

pub enum FileMode {
    SingleMode { length: usize },
    MultiMode { files: Vec<Files> },
}

pub struct Files {
    pub length: usize,
    pub path: Vec<String>,
}

impl Torrent {
    pub fn from_str(val: &str) {
        let mut u8s = bencode::utils::bcode_to_u8(val);
        let mut announce: String;
        while let Ok(b) = bencode::bencode::decode(&mut u8s) {
            match b {
                BTypes::DICT(d) => d.into_iter().for_each(|(k, v)| match k.as_str() {
                    "announce" => announce = v.try_into(),
                }),
                _ => panic!("NOT A TORRENT FILE"),
            }
        }
    }
}
