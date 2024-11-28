pub enum Message {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32),
    Unrecognized,
    Request,
}

pub enum DownloadError {
    Unrecognized,
}

impl TryFrom<&[u8]> for Message {
    type Error = DownloadError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let length = u32::from_be_bytes(value[0..4].try_into().unwrap());
        if length == 0 {
            return Ok(Self::KeepAlive);
        }
        let message = match (length, value[5]) {
            (1, 0) => Self::Choke,
            (1, 1) => Self::Unchoke,
            (1, 2) => Self::Interested,
            (1, 3) => Self::NotInterested,
            (5, 4) => Self::Have(u32::from_be_bytes(value[6..10].try_into().unwrap())),
            (13, 6) => Self::Request,
            _ => todo!(),
        };
        Ok(message)
    }
}
