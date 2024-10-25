use std::collections::BTreeMap;

use crate::error;
use bencode;
use bencode::bencode::BTypes;
use bencode::error::DecodeError;
use bencode::utils::decode_option;
use error::TorrentError;

#[derive(Default, Debug)]
pub struct Torrent {
    pub announce: String,
    pub announce_list: Option<Vec<String>>,
    pub creation_date: Option<usize>,
    pub comment: Option<String>,
    pub created_by: Option<String>,
    pub encoding: Option<String>,
    pub info: Info,
}

#[derive(Default, Debug)]
pub struct Info {
    pub name: String,
    pub piece_length: usize,
    pub pieces: Vec<u8>,
    pub mode: FileMode,
}

#[derive(Debug, PartialEq)]
pub enum FileMode {
    SingleMode { length: usize },
    MultiMode { files: Vec<Files> },
}

impl Default for FileMode {
    fn default() -> Self {
        Self::SingleMode { length: 0 }
    }
}

#[derive(Debug, PartialEq)]
pub struct Files {
    pub length: usize,
    pub path: Vec<String>,
}

impl Torrent {
    pub fn from_file(fs: &str) -> Result<Self, TorrentError> {
        use std::{fs::File, io::Read};
        let mut file = File::open(fs).unwrap();
        let mut content = vec![];
        file.read_to_end(&mut content).unwrap();
        let mut u8s = content.into_iter();
        Self::decode(&mut u8s)
    }
    pub fn from_str(val: &str) -> Result<Self, TorrentError> {
        let mut u8s = bencode::utils::bcode_to_u8(val);
        Self::decode(&mut u8s)
    }

    fn decode<T>(u8s: &mut T) -> Result<Self, TorrentError>
    where
        T: Iterator<Item = u8>,
    {
        let mut torrent = Self::default();
        if let Ok(b) = bencode::bencode::decode(u8s) {
            if let BTypes::DICT(d) = b {
                torrent.de_fields(d).expect("Failed to decrypt fields");
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
        self.announce_list = decode_option(d.get("announce-list"))?;
        self.creation_date = decode_option(d.get("creation date"))?;
        self.comment = decode_option(d.get("comment"))?;
        self.created_by = decode_option(d.get("created by"))?;
        self.encoding = decode_option(d.get("encoding"))?;
        self.de_info_fields(d.get("info"))?;
        Ok(())
    }

    fn de_info_fields(&mut self, d: Option<&BTypes>) -> Result<(), DecodeError> {
        if let Some(BTypes::DICT(d)) = d {
            self.info.name = d.get("name").unwrap().try_into()?;
            self.info.piece_length = d.get("piece length").unwrap().try_into()?;
            self.info.pieces = d.get("pieces").unwrap().try_into()?;
            if let Some(p) = d.get("files") {
                self.info.mode = Self::de_multi_file_mode(p)?;
            } else {
                self.info.mode = FileMode::SingleMode {
                    length: d.get("length").unwrap().try_into()?,
                }
            }
        } else {
            return Err(DecodeError::EOF);
        };
        Ok(())
    }

    fn de_multi_file_mode(d: &BTypes) -> Result<FileMode, DecodeError> {
        if let BTypes::LIST(l) = d {
            let files: Vec<Files> = l
                .iter()
                .filter_map(|d| {
                    if let BTypes::DICT(dict) = d {
                        Some(Files {
                            length: dict.get("length").unwrap().try_into().unwrap(),
                            path: dict.get("path").unwrap().try_into().unwrap(),
                        })
                    } else {
                        None
                    }
                })
                .collect();
            Ok(FileMode::MultiMode { files })
        } else {
            Err(DecodeError::EOF)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::torrent::FileMode;

    use super::Torrent;

    #[test]
    fn debain() {
        let fs = "debian.torrent";
        let torrent = Torrent::from_file(fs).unwrap();
        assert_eq!(
            torrent.announce,
            "http://bttracker.debian.org:6969/announce"
        );
        assert_eq!(
            torrent.comment,
            Some("\"Debian CD from cdimage.debian.org\"".to_string())
        );
        assert_eq!(torrent.created_by, Some("mktorrent 1.1".to_string()));
        assert_eq!(torrent.creation_date, Some(1725105953));
        assert_eq!(torrent.info.piece_length, 262144);
        assert_eq!(torrent.info.name, "debian-12.7.0-amd64-netinst.iso");
        assert_eq!(
            torrent.info.mode,
            FileMode::SingleMode { length: 661651456 }
        );
    }
}
