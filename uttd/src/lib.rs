pub mod error;
pub mod url;
pub mod urutil;

use std::{
    io::{Read, Write},
    net::{AddrParseError, TcpStream, UdpSocket},
    time::Duration,
};

use url::{Scheme, Url};

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

pub struct Stream<'a> {
    stream: StreamType,
    url: Url<'a>,
}

pub enum StreamType {
    TCP(TcpStream),
    UDP(UdpSocket),
}

impl<'a> Stream<'a> {
    pub fn new(url: Url<'a>) -> Result<Self, UttdError> {
        let stream = match url.scheme {
            Scheme::HTTP => StreamType::TCP(TcpStream::connect(url.socket).unwrap()),
            Scheme::UDP => {
                println!("{}", url.socket);
                let sock = UdpSocket::bind("0.0.0.0:0").unwrap();
                sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                sock.set_write_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                sock.connect(url.socket).unwrap();
                StreamType::UDP(sock)
            }
            _ => unimplemented!(),
        };
        // let stream = TcpStream::connect(socket)?;
        Ok(Stream { stream, url })
    }
    pub fn send(&mut self, data: &[u8]) -> Result<Vec<u8>, UttdError> {
        let mut res = vec![];
        match &mut self.stream {
            StreamType::TCP(t) => {
                t.write_all(data)?;
                t.read_to_end(&mut res)?;
            }
            StreamType::UDP(t) => {
                t.send(data)?;
                t.recv(&mut res)?;
            }
        }
        Ok(res)
    }
    pub fn get(&mut self, path: String) -> Result<Vec<u8>, UttdError> {
        println!("{}", self.url.host);
        let get_header = format!(
            "GET {} HTTP/1.1\r\n
            Host: {}\r\n
            Connection: close\r\n
            User-agent: torain\r\n
            Accept: */*\r\n
        ",
            path, self.url.host
        );
        let res = self.send(get_header.as_bytes())?;
        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use crate::{url::Url, Stream};
    use std::net::UdpSocket;

    // request google with bogus data

    #[test]
    fn http_get_request() {
        let url = Url::new("http://bttracker.debian.org:6969/announce").unwrap();
        let mut stream = Stream::new(url).unwrap();
        let response = stream.get("/".to_owned()).unwrap();

        assert!(!response.is_empty());
    }

    #[test]
    fn udp_get_request() {
        let url = Url::new("udp://open.stealth.si:80").unwrap();
        let mut stream = Stream::new(url).unwrap();
        let response = stream.get("/announce".to_owned()).unwrap();

        assert!(!response.is_empty());
    }
    #[test]
    fn raw_udp() {
        let stream = UdpSocket::bind("0.0.0.0:0").unwrap();
        let udp_sock = "1.1.1.1:53";
        let message = b"\x12\x34\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00\x03\x77\x77\x77\x06\x67\x6f\x6f\x67\x6c\x65\x03\x63\x6f\x6d\x00\x00\x01\x00\x01";
        stream.send_to(message, udp_sock).unwrap();
        let mut buf = [0; 1024];
        let (l, _) = stream.recv_from(&mut buf).unwrap();
        assert_eq!(l, 48);
    }
}
