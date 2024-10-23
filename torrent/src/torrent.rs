use std::collections::BTreeMap;

use crate::error;
use bencode;
use bencode::bencode::BTypes;
use bencode::error::DecodeError;
use bencode::utils;
use error::TorrentError;

#[derive(Default)]
pub struct Torrent {
    pub announce: String,
    pub announce_list: Option<Vec<String>>,
    pub creation_date: Option<i32>,
    pub comment: Option<String>,
    pub created_by: Option<String>,
    pub encoding: Option<String>,
    pub info: Info,
}

#[derive(Default)]
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

impl Default for FileMode {
    fn default() -> Self {
        Self::SingleMode { length: 0 }
    }
}

pub struct Files {
    pub length: usize,
    pub path: Vec<String>,
}

impl Torrent {
    pub fn from_str(val: &str) -> Result<Self, TorrentError> {
        let mut torrent = Self::default();
        let mut u8s = bencode::utils::bcode_to_u8(val);
        if let Ok(b) = bencode::bencode::decode(&mut u8s) {
            if let BTypes::DICT(d) = b {
                torrent.de_fields(d);
            } else {
                return Err(TorrentError::UnexpectedField);
            }
        } else {
            return Err(TorrentError::UnexpectedField);
        }
        Ok(torrent)
    }

    fn de_fields(&mut self, d: BTreeMap<String, BTypes>) -> Result<(), DecodeError> {
        self.announce = d.get("announce").unwrap().try_into()?;
        Ok(())
    }

    // fn de_info_fields(&mut self, k: String, v: BTypes) -> Result<(), DecodeError> {
    //     match k.to_str() {
    //         "name" => self.info.name = v.try_into()?,
    //     }
    // }
}
