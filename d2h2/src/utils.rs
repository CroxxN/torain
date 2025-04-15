use bencode::bencode::BTypes;

pub fn get_string_from_dict(bcode: &BTypes, key: &str) -> Option<String> {
    if let BTypes::DICT(k) = bcode {
        if let Some(v) = k.get(key) {
            return Some(v.try_into().unwrap());
        }
    }
    None
}

pub fn get_usize_from_dict(bcode: &BTypes, key: &str) -> Option<usize> {
    if let BTypes::DICT(k) = bcode {
        if let Some(v) = k.get(key) {
            return Some(v.try_into().unwrap());
        }
    }
    None
}
