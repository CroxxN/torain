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
    FailedRequest,
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
    pub stream: StreamType,
    host: String,
}

pub enum StreamType {
    TCP(TcpStream),
    UDP(Udp),
}

#[allow(dead_code)]
pub struct Udp {
    socket: UdpSocket,
    pub connection_id: i64,
}

impl Stream {
    /// Create a new Tcp or Udp stream on a Url
    /// The type of stream is based on the scheme in the url

    /// ```
    /// use uttd::url::Url;
    /// use uttd::Stream;
    /// let url = Url::new("http://google.com:80/some_page").unwrap();
    /// let mut request = Stream::new(&url).unwrap();
    /// let res = request.get("/").unwrap();
    /// assert!(!res.is_empty());
    /// ```
    pub fn new(url: &Url) -> Result<Self, UttdError> {
        let stream = match url.scheme {
            Scheme::HTTP => StreamType::TCP(TcpStream::connect(&url.url).unwrap()),
            Scheme::UDP => {
                let mut sock = UdpSocket::bind("0.0.0.0:0").unwrap();
                sock.set_read_timeout(Some(Duration::from_secs(5)))?;
                sock.set_write_timeout(Some(Duration::from_secs(5)))?;
                sock.connect(&url.url).unwrap();
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
            host: url.host.clone(),
        })
    }

    pub fn initiate_udp(stream: &mut UdpSocket) -> Result<i64, UttdError> {
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
        let connection_id = i64::from_be_bytes(res[8..16].try_into().unwrap());
        Ok(connection_id)
    }

    pub fn send(&mut self, data: &[u8], res: &mut Vec<u8>) -> Result<(), UttdError> {
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
        // if timeout, retry for 5 times
        for _ in 0..5 {
            stream.send(data).unwrap();
            if let Ok(_) = stream.recv(res) {
                break;
            }
        }
        // if after 10 tires still no response, return error
        if res.is_empty() {
            return Err(UttdError::FailedRequest);
        }

        Ok(())
    }

    /// Perform a get request on this stream
    /// `path` referes to the location of the url + any params
    /// For example: google.com:80/{path}?param=value

    /// ```
    /// use uttd::url::Url;
    /// use uttd::Stream;
    /// let url = Url::new("http://google.com:80/some_page").unwrap();
    /// let mut request = Stream::new(&url).unwrap();
    /// let res = request.get("/").unwrap();
    /// assert!(!res.is_empty());
    /// ```

    pub fn get(&mut self, path: &str) -> Result<Vec<u8>, UttdError> {
        let (host, _) = self.host.split_once(':').unwrap();
        let get_header = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            path, host
        );
        let mut res = vec![];
        self.send(get_header.as_bytes(), &mut res)?;
        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use crate::{url::Url, Stream, StreamType};
    use std::{
        io::{Read, Write},
        net::TcpStream,
    };

    // request google with bogus data

    #[test]
    fn http_get_request() {
        let url = Url::new("http://bttracker.debian.org:6969/announce").unwrap();
        println!("{}", url.url);
        let mut stream = Stream::new(&url).unwrap();
        let response = stream.get("/").unwrap();

        assert!(!response.is_empty());
    }

    #[test]
    fn udp_get_request() {
        let url = Url::new("udp://tracker.opentrackr.org:1337").unwrap();
        let stream = Stream::new(&url).unwrap();
        let mut res = 0;
        if let StreamType::UDP(u) = stream.stream {
            res = u.connection_id;
        }

        assert!(res != 0);
    }

    #[test]
    fn get_request() {
        let request = "GET /announce?uploaded=0&info_hash=%1b%d0%88%ee%91%66%a0%62%cf%4a%f0%9c%f9%97%20%fa%6e%1a%31%33&port=6881&peer_id=--sd--TORAIN---01523&compact=0&left=661651456&event=started&downloaded=0 HTTP/1.1\r\nHost: bttracker.debian.org\r\nConnection: close\r\n\r\n";
        let mut tcp = TcpStream::connect("bttracker.debian.org:6969").unwrap();
        tcp.write_all(request.as_bytes()).unwrap();
        let mut res = vec![];
        tcp.read_to_end(&mut res).unwrap();
        assert_eq!([res[9], res[10], res[11]], [b'2', b'0', b'0']);
    }
}
