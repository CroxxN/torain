// encoding to bencode
use crate::bencode::BTypes;
use std::collections::BTreeMap;

pub fn ser(de: &BTypes) -> Vec<u8> {
    match de {
        BTypes::INT(i) => ser_int(*i),
        BTypes::BSTRING(bs) => ser_bstring(bs),
        BTypes::LIST(l) => ser_list(l),
        BTypes::DICT(d) => ser_dict(d),
    }
}

fn ser_int(i: i64) -> Vec<u8> {
    format!("i{}e", i).into()
}

fn ser_bstring(bs: &Vec<u8>) -> Vec<u8> {
    let mut parsed = vec![];
    let len_str = format!("{}", bs.len()).into_bytes();
    parsed.extend_from_slice(&len_str);
    parsed.push(b':');
    parsed.extend(bs);
    parsed
}

fn ser_string(str: &String) -> Vec<u8> {
    let mut acc = vec![];
    let bytes = str.as_bytes();
    let len_str = format!("{}", bytes.len()).into_bytes();
    acc.extend(len_str);
    acc.push(b':');
    acc.extend(bytes);
    acc
}

fn ser_list(l: &Vec<BTypes>) -> Vec<u8> {
    let mut acc = vec![];
    acc.push(b'l');
    for bt in l {
        acc.extend(ser(bt));
    }
    acc.push(b'e');
    acc
}

fn ser_dict(d: &BTreeMap<String, BTypes>) -> Vec<u8> {
    let mut acc = vec![];
    acc.push(b'd');
    for (k, v) in d {
        acc.extend(ser_string(k));
        acc.extend(ser(v));
    }
    acc.push(b'e');
    acc
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use crate::{
        bencode::BTypes,
        bencoen::{ser_dict, ser_list, ser_string},
    };

    use super::ser;

    #[test]
    fn int() {
        assert_eq!("i64e".to_owned().into_bytes(), ser(&BTypes::INT(64)))
    }

    #[test]
    fn bstring() {
        assert_eq!(
            "4:type".to_owned().into_bytes(),
            ser(&BTypes::BSTRING("type".to_owned().into_bytes()))
        )
    }

    #[test]
    fn str() {
        assert_eq! {
            "4:type".to_owned().into_bytes(),
            ser_string(&"type".to_owned())
        }
    }

    #[test]
    fn list() {
        let mut list = vec![];
        list.push(BTypes::INT(4));
        list.push(BTypes::BSTRING("type".to_owned().into_bytes()));

        assert_eq!("li4e4:typee".to_owned().into_bytes(), ser_list(&list))
    }

    #[test]
    fn dict() {
        let mut dict = BTreeMap::new();
        dict.insert("info".into(), BTypes::INT(4));
        dict.insert("name".into(), BTypes::INT(42));
        assert_eq!(
            "d4:infoi4e4:namei42ee".to_owned().into_bytes(),
            ser_dict(&dict)
        )
    }

    #[test]
    fn empty_dict() {
        let dict: BTreeMap<String, BTypes> = BTreeMap::new();
        assert_eq!("de".to_owned().into_bytes(), ser_dict(&dict))
    }

    #[test]
    fn raw_bin() {
        let str = BTypes::BSTRING("þ".to_owned().into_bytes());
        assert_eq!(vec![b'2', b':', 195, 190], ser(&str));
    }
}
