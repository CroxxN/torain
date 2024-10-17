use core::panic;

use bencode;
use bencode::bencode::BTypes;
use error::TorrentError;

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
    pub fn from_str(val: &str) -> Result<Self, TorrentError> {
        let mut torrent = Self::default();
        let mut u8s = bencode::utils::bcode_to_u8(val);
        while let Ok(b) = bencode::bencode::decode(&mut u8s) {
            match b {
                BTypes::DICT(d) => d.into_iter().for_each(|(k, v)| self.de_fields(k, v)?),
                _ => Err(TorrentError::UnexpectedField)
            }
        }
        Ok(torrent)
    }

    fn de_fields(&mut self, k: String, v: BTypes)-> Result<(), DecodeError> {
        match k.to_str() {
            "announce" => self.announce = v.try_into()?,
            "announce-list" => self.announce_list = v.try_into()?,
            "creation date" => self.creation_date = v.try_into()?,
            "comment" => self.comment = v.try_into()?,
            "created by" => self.created_by = v.try_into()?,
            "encoding" => self.encoding = v.try_into()?,
            "info" => {
                if let BTypes::DICT(d) = v {
                    d.into_inter().for_each(|(k, v)| self.de_info_fields(k, v));
                } else {
                    Err(TorrentError::UnexpectedField);
                }
            },
            _ => Err(TorrentError::UnexpectedField)
        }
    }

    fn de_info_fields(&mut self, k: String, v: BTypes) -> Result<(), DecodeError>{
        match k.to_str() {
            "name" => self.info.name = v.try_into()?,
        }
            
    }
}
