pub enum BTypes {
    BSTRING,
    INT,
    LIST,
    DICT,
    UNKNOWN,
}

pub enum DecodeError {
    EOF,
    IntParseError,
}

fn bcode_to_u8<'a>(bcode: &'a str) {
    let mut u8s = bcode.bytes().map(|b| b);
    decode(&mut u8s);
}

pub fn decode<T>(data: &mut T) -> Result<(), DecodeError>
where
    T: Iterator<Item = u8>,
{
    let res = match data.next() {
        Some(t) => Ok(handle_data_type(data, t)),
        None => Err(DecodeError::EOF),
    };
    res
}

pub fn handle_data_type<T>(data: &mut T, anchor: u8)
where
    T: Iterator<Item = u8>,
{
    match anchor {
        b'i' => bcode_interger(data),
        _ => unimplemented!(),
    }
}

fn bcode_interger<T>(int_seq: &mut T) -> Result<i64, DecodeError>
where
    T: Iterator<Item = u8>,
{
    let holder: Vec<u8> = Vec::new();
    let int_token = match int_seq.next() {
        Some(t) => t,
        None => return Err(DecodeError::EOF),
    };
    if int_token == b'e' {
        let inter_string = vec_to_string(holder);
        return string_to_int(inter_string);
    }
    Ok(())
}

fn vec_to_string(holder: Vec<u8>) -> String {
    let vecstring = holder.iter().map(|&t| t as char).collect::<String>();
    vecstring
}

fn string_to_int(init: String) -> Result<i64, std::num::ParseIntError> {
    if Ok(v) = init.parse::<i64>() {
        (v)
    } else {
        DecodeError::IntParseError
    }
}
