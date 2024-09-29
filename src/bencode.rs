use std::collections::BTreeMap;

#[derive(PartialEq, Eq)]
pub enum BTypes {
    BSTRING(String),
    INT(i64),
    LIST(Vec<BTypes>),
    DICT(BTreeMap<String, BTypes>),
}

#[derive(Debug)]
pub enum DecodeError {
    EOF,
    IntParseError,
}

pub fn decode<T>(data: &mut T)
where
    T: Iterator<Item = u8>,
{
    while let Some(t) = data.next() {
        if let Ok(r) = handle_data_type(data, t) {
            publish_btypes(r)
        }
    }
}

fn publish_btypes(b: BTypes) {
    match b {
        BTypes::INT(i) => println!("{}", i),
        BTypes::BSTRING(s) => println!("{}", s),
        BTypes::LIST(l) => l.into_iter().for_each(|d| publish_btypes(d)),
        BTypes::DICT(d) => {
            d.into_iter().for_each(|(k, v)| {
                print!("{}: ", k);
                publish_btypes(v);
            });
        }
    }
}

pub fn handle_data_type<T>(data: &mut T, anchor: u8) -> Result<BTypes, DecodeError>
where
    T: Iterator<Item = u8>,
{
    match anchor {
        b'i' => Ok(BTypes::INT(bcode_interger(data)?)),
        b'l' => Ok(BTypes::LIST(bcode_list(data)?)),
        b'd' => Ok(BTypes::DICT(bcode_dict(data)?)),
        b'0'..b'9' => Ok(BTypes::BSTRING(bcode_string(data, anchor)?)),
        _ => Ok(BTypes::BSTRING("".to_string())),
    }
}

fn bcode_interger<T>(int_seq: &mut T) -> Result<i64, DecodeError>
where
    T: Iterator<Item = u8>,
{
    let mut holder: Vec<u8> = Vec::new();

    loop {
        match int_seq.next() {
            Some(t) => {
                if t == b'e' {
                    break;
                }
                holder.push(t)
            }
            None => return Err(DecodeError::EOF),
        };
    }
    let inter_string = vec_to_string(holder);

    string_to_int(inter_string)
        .map(|res| res)
        .map_err(|_| DecodeError::IntParseError)
}

fn vec_to_string(holder: Vec<u8>) -> String {
    if holder.is_empty() {
        return "0".to_string();
    }
    let vecstring = holder.iter().map(|&t| t as char).collect::<String>();
    vecstring
}

fn string_to_int(init: String) -> Result<i64, DecodeError> {
    if let Ok(v) = init.parse::<i64>() {
        Ok(v)
    } else {
        Err(DecodeError::IntParseError)
    }
}

fn bcode_string<T>(str_seq: &mut T, anchor: u8) -> Result<String, DecodeError>
where
    T: Iterator<Item = u8>,
{
    let mut vec_u8: Vec<u8> = Vec::new();
    vec_u8.push(anchor);
    vec_u8.extend(str_seq.take_while(|c| *c != b':'));
    let length = string_to_int(vec_to_string(vec_u8))? as usize;
    let str_u8: Vec<u8> = str_seq.take(length).collect();
    Ok(vec_to_string(str_u8))
}

fn bcode_list<T>(list_seq: &mut T) -> Result<Vec<BTypes>, DecodeError>
where
    T: Iterator<Item = u8>,
{
    let mut holder = vec![];
    while let Some(anchor) = list_seq.next() {
        if anchor == b'e' {
            return Ok(holder);
        }
        holder.push(handle_data_type(list_seq, anchor)?);
    }
    Ok(holder)
}

fn bcode_dict<T>(d_seq: &mut T) -> Result<BTreeMap<String, BTypes>, DecodeError>
where
    T: Iterator<Item = u8>,
{
    let mut hmap = BTreeMap::new();
    while let Some(anchor) = d_seq.next() {
        if anchor == b'e' {
            return Ok(hmap);
        }
        let bt = handle_data_type(d_seq, anchor)?;
        if let BTypes::BSTRING(s) = bt {
            if let Some(anchor) = d_seq.next() {
                hmap.insert(s, handle_data_type(d_seq, anchor)?);
            }
        }
    }
    Ok(hmap)
}
