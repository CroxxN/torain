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

pub struct Stream {
    stream: StreamType,
    host: String,
}

pub enum StreamType {
    TCP(TcpStream),
    UDP(Udp),
}

pub struct Udp {
    socket: UdpSocket,
    connection_id: u64,
}

impl Stream {
    pub fn new(url: Url) -> Result<Self, UttdError> {
        let stream = match url.scheme {
            Scheme::HTTP => StreamType::TCP(TcpStream::connect(url.url).unwrap()),
            Scheme::UDP => {
                let mut sock = UdpSocket::bind("0.0.0.0:0").unwrap();
                sock.connect(url.url).unwrap();
                let connection_id = Self::initiate_udp(&mut sock)?;
                StreamType::UDP(Udp {
                    socket: sock,
                    connection_id,
                })
            }
            _ => unimplemented!(),
        };
        Ok(Stream {
            stream,
            host: url.host,
        })
    }

    pub fn initiate_udp(stream: &mut UdpSocket) -> Result<u64, UttdError> {
        let protocol_id: i64 = 0x41727101980; // Protocol ID
        let action: i32 = 0; // Action: connect
        let transaction_id: i32 = 1; // Random Transaction ID

        let mut buf = Vec::new();
        buf.extend_from_slice(&protocol_id.to_be_bytes());
        buf.extend_from_slice(&action.to_be_bytes());
        buf.extend_from_slice(&transaction_id.to_be_bytes());
        let mut res = vec![0; 16];
        Self::send_udp(stream, &buf, &mut res)?;
        // a UDP initiate response is always 16 bytes
        // https://www.bittorrent.org/beps/bep_0029.html
        let connection_id = u64::from_be_bytes(res[8..16].try_into().unwrap());
        Ok(connection_id)
    }

    fn send(&mut self, data: &[u8], res: &mut Vec<u8>) -> Result<(), UttdError> {
        // Using Vec::new() works for tcp streams but fails for UDP requests because the .recv()
        // method for UDP expects an already allocated buffer. Vec::new() just creates a container with lenght 0.
        // So, we iniliatize a vec with vec![] to initialize a vec with 1024 bytes of space. If any request is larger than that,
        // the vec accomodates to fill the space.

        // let mut res = vec![0u8; 1024];
        // let mut udp_res = [0; 1024];
        match &mut self.stream {
            StreamType::TCP(t) => {
                Self::send_tcp(t, data, res)?;
            }
            StreamType::UDP(t) => {
                Self::send_udp(&mut t.socket, data, res)?;
            }
        }
        Ok(())
    }

    fn send_tcp(stream: &mut TcpStream, data: &[u8], res: &mut Vec<u8>) -> Result<(), UttdError> {
        stream.write_all(data)?;
        stream.read_to_end(res)?;
        Ok(())
    }

    fn send_udp(stream: &mut UdpSocket, data: &[u8], res: &mut Vec<u8>) -> Result<(), UttdError> {
        stream.send(data)?;
        stream.recv(res)?;
        Ok(())
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
        let mut res = vec![];
        self.send(get_header.as_bytes(), &mut res)?;
        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use crate::{url::Url, Stream, StreamType};
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
        let stream = Stream::new(url).unwrap();
        let mut res = 0;
        if let StreamType::UDP(u) = stream.stream {
            res = u.connection_id;
        }

        assert!(res != 0);
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
