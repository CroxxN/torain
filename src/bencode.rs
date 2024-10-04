use std::{collections::BTreeMap, string::FromUtf8Error};

#[derive(PartialEq, Eq)]
pub enum BTypes {
    BSTRING(Vec<u8>),
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
            _ = publish_btypes(r);
        }
    }
}

fn publish_btypes(b: BTypes) -> Result<(), FromUtf8Error> {
    match b {
        BTypes::INT(i) => println!("{}", i),
        BTypes::BSTRING(b) => {
            if let Ok(s) = String::from_utf8(b.clone()) {
                println!("{}", s);
            } else {
                b.into_iter().for_each(|b| print!("{:#02x} ", b));
                println!();
            }
        }
        BTypes::LIST(l) => l.into_iter().for_each(|d| _ = publish_btypes(d)),
        BTypes::DICT(d) => {
            d.into_iter().for_each(|(k, v)| {
                print!("{}: ", k);
                _ = publish_btypes(v);
            });
        }
    };
    Ok(())
}

pub fn handle_data_type<T>(data: &mut T, anchor: u8) -> Result<BTypes, DecodeError>
where
    T: Iterator<Item = u8>,
{
    match anchor as char {
        'i' => Ok(BTypes::INT(bcode_interger(data)?)),
        'l' => Ok(BTypes::LIST(bcode_list(data)?)),
        'd' => Ok(BTypes::DICT(bcode_dict(data)?)),
        '0'..='9' => Ok(BTypes::BSTRING(bcode_string(data, anchor)?)),
        _ => {
            println!("Error: Unknown character '{}' found", anchor);
            Err(DecodeError::EOF)
        }
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

fn bcode_string<T>(str_seq: &mut T, anchor: u8) -> Result<Vec<u8>, DecodeError>
where
    T: Iterator<Item = u8>,
{
    let mut vec_u8: Vec<u8> = Vec::new();
    vec_u8.push(anchor);
    vec_u8.extend(str_seq.take_while(|c| *c != b':'));
    let length = string_to_int(vec_to_string(vec_u8))? as usize;
    let str_u8: Vec<u8> = str_seq.take(length).collect();
    Ok(str_u8)
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
                hmap.insert(vec_to_string(s), handle_data_type(d_seq, anchor)?);
            }
        }
    }
    Ok(hmap)
}
