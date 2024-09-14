pub enum BTypes {
    BSTRING,
    INT,
    LIST,
    DICT,
    UNKNOWN,
}

#[derive(Debug)]
pub enum DecodeError {
    EOF,
    IntParseError,
}

pub fn bcode_to_u8<'a>(bcode: &'a str) -> i64 {
    let mut u8s = bcode.bytes().map(|b| b);
    decode(&mut u8s).expect("Failed")
}

pub fn decode<T>(data: &mut T) -> Result<i64, DecodeError>
where
    T: Iterator<Item = u8>,
{
    let res = match data.next() {
        Some(t) => Ok(handle_data_type(data, t)),
        None => Err(DecodeError::EOF),
    };
    res
}

pub fn handle_data_type<T>(data: &mut T, anchor: u8) -> i64
where
    T: Iterator<Item = u8>,
{
    match anchor {
        b'i' => bcode_interger(data).expect("Some typa error"),
        _ => unimplemented!(),
    }
}

fn bcode_interger<T>(int_seq: &mut T) -> Result<i64, DecodeError>
where
    T: Iterator<Item = u8>,
{
    let mut holder: Vec<u8> = Vec::new();
    let mut sign = 1i8;
    loop {
        match int_seq.next() {
            Some(t) => {
                if t == b'e' {
                    break;
                }
                if t == b'-' {
                    sign = -1;
                    continue;
                }
                holder.push(t)
            }
            None => return Err(DecodeError::EOF),
        };
    }
    let inter_string = vec_to_string(holder);
    if let Ok(t) = string_to_int(inter_string) {
        Ok(t * sign as i64)
    } else {
        Err(DecodeError::IntParseError)
    }
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
