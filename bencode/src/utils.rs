use crate::bencode::BTypes;

pub enum BencodeErr {
    Berr,
}

pub fn bcode_to_u8(bcode: &str) -> impl Iterator<Item = u8> + '_ {
    bcode.bytes()
}

pub fn vec_to_string(holder: &Vec<u8>) -> String {
    let vecstring = holder.iter().map(|&t| t as char).collect::<String>();
    vecstring
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
