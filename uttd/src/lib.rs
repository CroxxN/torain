mod tracker;
pub mod urutil;

use std::{
    io::{Read, Write},
    net::{AddrParseError, SocketAddr, TcpStream, ToSocketAddrs},
};

#[derive(Debug)]
pub enum UttdError {
    IpParseFail(AddrParseError),
    IoError(std::io::Error),
}

impl From<AddrParseError> for UttdError {
    fn from(value: AddrParseError) -> Self {
        Self::IpParseFail(value)
    }
}

impl From<std::io::Error> for UttdError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

pub struct Url {
    stream: TcpStream,
}

impl Url {
    pub fn new(address: &str, port: u16) -> Result<Self, UttdError> {
        let mut sock_adr = (address, port).to_socket_addrs()?;
        let ip = sock_adr.next().unwrap();
        let socket = SocketAddr::from(ip);
        let stream = TcpStream::connect(socket)?;
        Ok(Url { stream })
    }
    pub fn send(&mut self, data: &[u8]) -> Result<Vec<u8>, UttdError> {
        let mut res = vec![];
        self.stream.write_all(data)?;
        self.stream.read_to_end(&mut res)?;
        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use crate::Url;

    // request google with bogus data
    #[test]
    fn get() {
        let mut url = Url::new("google.com", 443).unwrap();
        let response = url.send(&[0]).unwrap();

        assert_eq!(response, "".to_owned().as_bytes());
    }
}
