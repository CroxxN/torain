mod tracker;
mod urutil;

use std::{
    net::{AddrParseError, IpAddr, SocketAddr},
    str::FromStr,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

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
    pub async fn new(address: &str, port: u16) -> Result<Self, UttdError> {
        let ip = IpAddr::from_str(address)?;
        let sock_addr = SocketAddr::new(ip, port);
        let stream = tokio::net::TcpStream::connect(sock_addr).await?;
        Ok(Url { stream })
    }
    async fn send(&mut self, data: &[u8]) -> Result<&mut [u8], UttdError> {
        let mut res: &mut [u8] = &mut [];
        self.stream.write_all(data).await?;
        self.stream.read_buf(&mut res).await?;
        Ok(res)
    }
}
