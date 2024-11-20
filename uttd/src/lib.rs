pub mod error;
pub mod url;
pub mod urutil;

use std::{
    io::{Read, Write},
    net::{AddrParseError, TcpStream, UdpSocket},
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
    host: &'a str,
}

pub enum StreamType {
    TCP(TcpStream),
    UDP(UdpSocket),
}

impl<'a> Stream<'a> {
    pub fn new(url: Url<'a>) -> Result<Self, UttdError> {
        let stream = match url.scheme {
            Scheme::HTTP => StreamType::TCP(TcpStream::connect(url.url).unwrap()),
            Scheme::UDP => {
                println!("{}", url.url);
                let sock = UdpSocket::bind("0.0.0.0:0").unwrap();
                // sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                // sock.set_write_timeout(Some(Duration::from_secs(5)))
                //     .unwrap();
                sock.connect(url.url).unwrap();
                // sock.
                StreamType::UDP(sock)
            }
            _ => unimplemented!(),
        };
        // let stream = TcpStream::connect(socket)?;
        Ok(Stream {
            stream,
            host: url.host,
        })
    }

    pub fn initiate_udp(&mut self) -> Result<Vec<u8>, UttdError> {
        let protocol_id: i64 = 0x41727101980; // Protocol ID
        let action: i32 = 0; // Action: connect
        let transaction_id: i32 = 1; // Random Transaction ID

        let mut buf = Vec::new();
        buf.extend_from_slice(&protocol_id.to_be_bytes());
        buf.extend_from_slice(&action.to_be_bytes());
        buf.extend_from_slice(&transaction_id.to_be_bytes());
        let res = self.send(&buf)?;
        Ok(res)
    }

    fn send(&mut self, data: &[u8]) -> Result<Vec<u8>, UttdError> {
        let mut res = vec![0u8; 16];
        // let mut udp_res = [0; 1024];
        match &mut self.stream {
            StreamType::TCP(t) => {
                t.write_all(data)?;
                t.read_to_end(&mut res)?;
            }
            StreamType::UDP(t) => {
                if let Err(e) = t.send(data) {
                    println!("{}", e);
                }
                t.recv(&mut res).unwrap();
                // t.recv(&mut res)?;
            }
        }
        Ok(res.to_vec())
    }
    pub fn get(&mut self, path: String) -> Result<Vec<u8>, UttdError> {
        let get_header = format!(
            "GET {} HTTP/1.1\r\n
            Host: {}\r\n
            Connection: close\r\n
            User-agent: torain\r\n
            Accept: */*\r\n
        ",
            path, self.host
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
        println!("{}", url.url);
        let mut stream = Stream::new(url).unwrap();
        let response = stream.get("/".to_owned()).unwrap();

        assert!(!response.is_empty());
    }

    #[test]
    fn udp_get_request() {
        let url = Url::new("udp://tracker.opentrackr.org:1337").unwrap();
        let mut stream = Stream::new(url).unwrap();
        let res = stream.initiate_udp().unwrap();
        // let response = stream.get("/".to_owned()).unwrap();

        // assert_eq!(
        //     response.iter().map(|x| *x as char).collect::<String>(),
        //     "".to_owned()
        // );
        assert!(!res.is_empty());
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
    #[test]
    fn raw_udp_tracker() {
        let stream = UdpSocket::bind("0.0.0.0:0").unwrap();
        let protocol_id: i64 = 0x41727101980; // Protocol ID
        let action: i32 = 0; // Action: connect
        let transaction_id: i32 = 1; // Random Transaction ID

        let mut buf = Vec::new();
        buf.extend_from_slice(&protocol_id.to_be_bytes());
        buf.extend_from_slice(&action.to_be_bytes());
        buf.extend_from_slice(&transaction_id.to_be_bytes());
        stream.connect("open.demonii.com:1337").unwrap();

        stream.send(&buf).unwrap();
        let mut buf = [0; 16];
        stream.recv(&mut buf).unwrap();
        assert_eq!(buf.len(), 16);
    }
}
