pub enum BTypes {
    BSTRING(String),
    INT(i64),
    LIST,
    DICT,
    UNKNOWN,
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
            match r {
                BTypes::INT(i) => println!("{}", i),
                BTypes::BSTRING(s) => println!("{}", s),
                _ => unimplemented!(),
            }
        }
    }
}

pub fn handle_data_type<T>(data: &mut T, anchor: u8) -> Result<BTypes, DecodeError>
where
    T: Iterator<Item = u8>,
{
    match anchor {
        b'i' => Ok(BTypes::INT(bcode_interger(data)?)),
        b'l' => unimplemented!(),
        b'd' => unimplemented!(),
        _ => Ok(BTypes::BSTRING(bcode_string(data, anchor)?)),
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
