use crate::error::UrlError;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Url<'a> {
    pub socket: SocketAddr,
    pub scheme: Scheme,
    pub host: &'a str,
    pub location: &'a str,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Scheme {
    HTTP,
    HTTPS,
    UDP,
}

impl From<&str> for Scheme {
    fn from(value: &str) -> Self {
        match value {
            "http" => Self::HTTP,
            "https" => Self::HTTPS,
            _ => Self::UDP,
        }
    }
}

impl<'a> Default for Url<'a> {
    fn default() -> Self {
        Self {
            socket: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            scheme: Scheme::UDP,
            host: "127.0.0.1",
            location: "/",
        }
    }
}

impl<'a> Url<'a> {
    pub fn new(address: &'a str) -> Result<Self, UrlError> {
        let (scheme, base) = address.split_once(':').ok_or(UrlError::InvalidUrl)?;
        let (base, port) = base.rsplit_once(':').ok_or(UrlError::InvalidUrl)?;
        let base = base.strip_prefix("//").ok_or(UrlError::InvalidUrl)?;

        let mut port = port;
        let mut location = "/";

        if let Some((p, loc)) = port.rsplit_once('/') {
            port = p;
            location = loc;
        }
        let port = port.parse::<u16>()?;
        let socket = (base, port)
            .to_socket_addrs()
            .expect("Failed to create socket")
            .next()
            .unwrap();

        Ok(Self {
            socket,
            scheme: scheme.into(),
            host: base,
            location,
        })
    }

    pub fn port(&self) -> u16 {
        self.socket.port()
    }
}

#[cfg(test)]
mod test {

    use crate::url::Scheme;

    use super::Url;

    #[test]
    fn http_announce() {
        let url = Url::new("http://bttracker.debian.org:6969/announce").unwrap();
        let port = url.port();
        let scheme = url.scheme;
        let host = url.host;
        let location = url.location;

        assert_eq!(
            (port, scheme, host, location),
            (6969, Scheme::HTTP, "bttracker.debian.org", "announce")
        );
    }

    #[test]
    fn udp_announce() {
        let url = Url::new("udp://open.demonii.com:1337").unwrap();

        let port = url.port();
        let scheme = url.scheme;
        let host = url.host;
        let location = url.location;

        assert_eq!(
            (port, scheme, host, location),
            (1337, Scheme::UDP, "open.demonii.com", "/")
        );
    }
}
