use crate::bencode::{self, BTypes};
use uttd::{error::UrlError, url::Url};

#[derive(Debug)]
pub enum BencodeErr {
    Berr,
    InvalidUrl,
}

pub fn bcode_to_u8(bcode: &str) -> impl Iterator<Item = u8> + '_ {
    bcode.bytes()
}

pub fn vec_to_string(holder: &Vec<u8>) -> String {
    let vecstring = holder.iter().map(|&t| t as char).collect::<String>();
    vecstring
}

pub fn decode_option<'a, T: TryFrom<&'a bencode::BTypes, Error = BencodeErr>>(
    value: Option<&'a BTypes>,
) -> Result<Option<T>, BencodeErr> {
    if let Some(v) = value {
        Ok(Some(v.try_into()?))
    } else {
        Ok(None)
    }
}

impl From<UrlError> for BencodeErr {
    fn from(_: UrlError) -> Self {
        Self::InvalidUrl
    }
}

impl TryFrom<&BTypes> for Url {
    type Error = BencodeErr;
    fn try_from(value: &BTypes) -> Result<Self, Self::Error> {
        if let BTypes::BSTRING(s) = value {
            let p = std::str::from_utf8(s).unwrap();
            Ok(Url::new(p)?)
        } else {
            Err(BencodeErr::Berr)
        }
    }
}

impl TryFrom<&BTypes> for String {
    type Error = BencodeErr;
    fn try_from(value: &BTypes) -> Result<Self, Self::Error> {
        if let BTypes::BSTRING(s) = value {
            Ok(vec_to_string(s))
        } else {
            Err(BencodeErr::Berr)
        }
    }
}

impl TryFrom<&BTypes> for usize {
    type Error = BencodeErr;
    fn try_from(value: &BTypes) -> Result<Self, Self::Error> {
        if let BTypes::INT(s) = value {
            Ok(*s as usize)
        } else {
            Err(BencodeErr::Berr)
        }
    }
}

impl TryFrom<&BTypes> for Vec<String> {
    type Error = BencodeErr;

    fn try_from(value: &BTypes) -> Result<Self, Self::Error> {
        let store: Vec<String>;
        if let BTypes::LIST(l) = value {
            store = l.iter().filter_map(|v| v.try_into().ok()).collect();
        } else {
            return Err(BencodeErr::Berr);
        }
        Ok(store)
    }
}

impl TryFrom<&BTypes> for Vec<u8> {
    type Error = BencodeErr;
    fn try_from(value: &BTypes) -> Result<Self, Self::Error> {
        if let BTypes::BSTRING(s) = value {
            Ok(s.to_vec())
        } else {
            Err(BencodeErr::Berr)
        }
    }
}
