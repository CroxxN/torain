pub enum BTypes {
    BSTRING,
    INT,
    LIST,
    DICT,
    UNKNOWN,
}

pub fn decode<'a>(data: &'a str) -> BTypes {
    let cursor = 0usize;
    match data.as_bytes()[cursor] {
        b'i' => return BTypes::INT,
        _ => return BTypes::UNKNOWN,
    };
}
