// encoding to bencode
use crate::bencode::BTypes;
use crate::utils::vec_to_string;
use std::collections::BTreeMap;

pub fn ser(de: BTypes) -> String {
    match de {
        BTypes::INT(i) => ser_int(i),
        BTypes::BSTRING(bs) => ser_bstring(bs),
        BTypes::LIST(l) => ser_list(l),
        BTypes::DICT(d) => ser_dict(d),
    }
}

fn ser_int(i: i64) -> String {
    format!("i{}e", i)
}

fn ser_bstring(bs: Vec<u8>) -> String {
    let parsed = vec_to_string(&bs);
    format!("{}:{}", parsed.len(), parsed)
}

fn ser_string(str: String) -> String {
    format!("{}:{}", str.len(), str)
}

fn ser_list(l: Vec<BTypes>) -> String {
    let mut acc = String::new();
    acc.push('l');
    for bt in l {
        acc.push_str(&ser(bt));
    }
    acc.push('e');
    acc
}

fn ser_dict(d: BTreeMap<String, BTypes>) -> String {
    let mut acc = String::new();
    acc.push('d');
    for (k, v) in d {
        acc.push_str(&ser_string(k));
        acc.push_str(&ser(v));
    }
    acc.push('e');
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
        assert_eq!("i64e", ser(BTypes::INT(64)))
    }

    #[test]
    fn bstring() {
        assert_eq!(
            "4:type",
            ser(BTypes::BSTRING("type".to_owned().into_bytes()))
        )
    }

    #[test]
    fn str() {
        assert_eq! {
            "4:type",
            ser_string("type".to_owned())
        }
    }

    #[test]
    fn list() {
        let mut list = vec![];
        list.push(BTypes::INT(4));
        list.push(BTypes::BSTRING("type".to_owned().into_bytes()));

        assert_eq!("li4e4:typee", ser_list(list))
    }

    #[test]
    fn dict() {
        let mut dict = BTreeMap::new();
        dict.insert("info".into(), BTypes::INT(4));
        dict.insert("name".into(), BTypes::INT(42));
        assert_eq!("d4:infoi4e4:namei42ee", ser_dict(dict))
    }

    #[test]
    fn empty_dict() {
        let dict: BTreeMap<String, BTypes> = BTreeMap::new();
        assert_eq!("de", ser_dict(dict))
    }

    // #[test]
    // fn raw_bin() {
    //     let str = "þ";

    // }
}
