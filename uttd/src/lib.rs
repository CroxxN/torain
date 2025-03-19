pub mod error;
pub mod url;
pub mod urutil;
pub mod utp;

use std::{
    io::{Read, Write},
    net::{AddrParseError, TcpStream, UdpSocket},
    time::Duration,
};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use url::{Scheme, Url};

#[derive(Debug)]
pub enum UttdError {
    IpParseFail(AddrParseError),
    IoError(std::io::Error),
    FailedRequest,
    RequestTimeout,
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

impl From<tokio::time::error::Elapsed> for UttdError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        Self::RequestTimeout
    }
}

// synchronous stream
pub struct Stream {
    pub stream: StreamType,
    host: String,
}

pub enum StreamType {
    TCP(TcpStream),
    UDP(Udp),
}

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
            Scheme::HTTP => StreamType::TCP(TcpStream::connect(&url.host).unwrap()),
            Scheme::UDP => {
                let mut sock = UdpSocket::bind("0.0.0.0:0").unwrap();
                sock.set_read_timeout(Some(Duration::from_secs(5)))?;
                sock.set_write_timeout(Some(Duration::from_secs(5)))?;
                sock.connect(&url.host).unwrap();
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
            host: url.host.to_owned(),
        })
    }

    /// UDP trackers require a initial handshake type message passing
    /// Defined in BEP 00015
    /// https://www.bittorrent.org/beps/bep_0015.html

    pub fn initiate_udp(stream: &mut UdpSocket) -> Result<i64, UttdError> {
        // magic
        let protocol_id: i64 = 0x41727101980; // Protocol ID
                                              // magic
        let action: i32 = 0; // Action: connect
                             // TODO: use rng
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

    /// Send `data` to `url` and return response in `res`
    /// Note: for UDP trackers, `res` must be initialized, preferably with 0's
    /// ```
    /// // will not work
    /// let mut res = Vec::new();
    /// // will work
    /// let mut res = vec![0; 10];
    /// // or
    /// let mut res = Vec::with_capacity(10);
    /// ```
    pub fn send(&mut self, data: &[u8], res: &mut Vec<u8>) -> Result<(), UttdError> {
        // Using Vec::new() works for tcp streams but fails for UDP requests because the .recv()
        // method for UDP expects an already allocated buffer. Vec::new() just creates a container with length 0.
        // So, we iniliatize a vec with vec![] to initialize a vec with 1024 bytes of space. If any request is larger than that,
        // the vec accommodates to fill the space.

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
        if res.len() == 0 {
            stream.write_all(data)?;
            stream.read_to_end(res)?;
        } else {
            stream.write_all(data).unwrap();
            assert!(res.len() == 68);
            stream.read_exact(res).unwrap();
        }
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
    /// `path` refers to the location of the url + any params
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
        let (host, _) = self.host.split_once(":").unwrap();

        let get_header = format!(
            "GET /{} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            path, host
        );
        let mut res = vec![];
        self.send(get_header.as_bytes(), &mut res)?;
        Ok(res)
    }
}

/// Async version of `Stream`
/// Holds a TcpStream underneth
#[derive(Debug)]
pub struct AsyncStream {
    pub async_stream_type: AsyncStreamType,
}

#[derive(Debug)]
pub enum AsyncStreamType {
    TcpStream(tokio::net::TcpStream),
    UtpStream(tokio::net::UdpSocket),
}

impl<'a> AsyncStream {
    pub async fn new(url: Url) -> Result<Self, UttdError> {
        match url.scheme {
            Scheme::HTTP => {
                let stream = tokio::time::timeout(
                    Duration::from_secs(5),
                    tokio::net::TcpStream::connect(&url.host),
                )
                .await??;
                Ok(AsyncStream {
                    async_stream_type: AsyncStreamType::TcpStream(stream),
                })
            }
            Scheme::UDP => {
                let stream = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
                stream.connect(&url.host).await?;
                Ok(AsyncStream {
                    async_stream_type: AsyncStreamType::UtpStream(stream),
                })
            }
            _ => unimplemented!(),
        }
    }

    /// Send `data` to the stream and receive in `res`
    /// Note: Peers are continuous stream of data. You must
    /// have initialized `res` with sufficient bytes. It only the exact bytes as is the capacity of `res`
    pub async fn send(&mut self, data: &[u8], res: &mut Vec<u8>) -> Result<usize, UttdError> {
        match self {
            AsyncStream {
                async_stream_type: AsyncStreamType::TcpStream(t),
            } => Self::send_tcp(t, data, res).await,
            AsyncStream {
                async_stream_type: AsyncStreamType::UtpStream(u),
            } => Self::send_utp(u, data, res).await,
        }
    }

    pub async fn send_tcp(
        tcp: &mut tokio::net::TcpStream,
        data: &[u8],
        res: &mut Vec<u8>,
    ) -> Result<usize, UttdError> {
        tcp.write_all(data).await.unwrap();
        let response = tokio::time::timeout(Duration::from_secs(15), tcp.read_exact(res)).await?;
        Ok(response?)
    }

    pub async fn send_utp(
        utp: &mut tokio::net::UdpSocket,
        data: &[u8],
        res: &mut Vec<u8>,
    ) -> Result<usize, UttdError> {
        let mut read = 0;
        for _ in 0..5 {
            read = utp.send(data).await.unwrap();
            if let Ok(r) = tokio::time::timeout(Duration::from_secs(10), utp.recv(res)).await? {
                read = r;
                break;
            }
        }
        // if after 10 tires still no response, return error
        if res.is_empty() {
            return Err(UttdError::FailedRequest);
        }

        Ok(read)
    }

    /// Read 4 bytes of data once and return
    pub async fn read_once(&mut self) -> Result<u32, UttdError> {
        match self {
            AsyncStream {
                async_stream_type: AsyncStreamType::TcpStream(t),
            } => Self::read_once_tcp(t).await,
            _ => unimplemented!(),
        }
    }

    async fn read_once_tcp(tcp: &mut tokio::net::TcpStream) -> Result<u32, UttdError> {
        // peers send keep_alive messages every 2 minutes. If we don't receive anything for 2 minutes, we close the connection
        let mut res = [0_u8; 4];
        _ = tokio::time::timeout(Duration::from_secs(121), tcp.read_exact(&mut res)).await??;
        let length = u32::from_be_bytes(res.try_into().unwrap());
        Ok(length)
    }

    pub async fn read_multiple(&mut self, res: &mut Vec<u8>) -> Result<(), UttdError> {
        match self {
            AsyncStream {
                async_stream_type: AsyncStreamType::TcpStream(t),
            } => Self::read_multiple_tcp(t, res).await,
            AsyncStream {
                async_stream_type: AsyncStreamType::UtpStream(u),
            } => Self::read_multiple_utp(u, res).await,
        }
    }

    // TODO: return the amount of bytes read
    /// Read `res.len()` bytes of data and pass it through `res`
    pub async fn read_multiple_tcp(
        tcp: &mut tokio::net::TcpStream,
        res: &mut Vec<u8>,
    ) -> Result<(), UttdError> {
        _ = tokio::time::timeout(Duration::from_secs(121), tcp.read_exact(res)).await??;
        Ok(())
    }

    pub async fn read_multiple_utp(
        utp: &mut tokio::net::UdpSocket,
        res: &mut Vec<u8>,
    ) -> Result<(), UttdError> {
        _ = tokio::time::timeout(Duration::from_secs(121), utp.recv(res)).await??;
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::{url::Url, utp::UtpPacket, AsyncStream, Stream, StreamType};
    use std::{
        io::{Read, Write},
        net::TcpStream,
    };

    // request google with bogus data

    #[test]
    fn http_get_request() {
        let url = Url::new("http://bttracker.debian.org:6969/announce").unwrap();
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
    // #[tokio::test]
    // async fn test_async_stream() {
    //     let fs = "debian.torrent";
    //     let torrent = Torrent::from_file(fs).unwrap();
    //     let tracker = TrackerParams::new(&torrent);
    //     let _announce = tracker.announce().unwrap();
    //     let info_hash = torrent.hash;
    //     let peer_id = tracker.peer_id;
    //     let mut handshake = peer::Handshake::new(info_hash, peer_id);

    //     let url = Url::new("https://google.com:80").unwrap();
    //     let mut stream = AsyncStream::handshake(&url, handshake).await.unwrap();
    //     let mut res = vec![0; 8];
    //     stream.send(&[0], &mut res).await.unwrap();
    //     assert!(res[0] != 0);
    // }

    // IMPORTANT: Can't depend on this specific peer being alive to may fail
    #[tokio::test]
    async fn test_utp_handshake() {
        let mut stream = tokio::net::UdpSocket::bind("0.0.0.0:0").await.unwrap();
        stream.connect("37.228.205.62:15223").await.unwrap();

        let bytes = UtpPacket::new().as_bytes();
        println!("{:?}", bytes);
        let mut res = vec![0; 20];

        let _ = AsyncStream::send_utp(&mut stream, &bytes, &mut res)
            .await
            .unwrap();
        assert_eq!(res[0], 33);
    }
}
