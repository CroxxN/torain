use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};

#[derive(PartialEq, Eq, Debug)]
pub struct Url {
    scheme: Scheme,
    url: SocketAddr,
}

#[derive(PartialEq, Eq, Debug)]
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

impl Default for Url {
    fn default() -> Self {
        Self {
            url: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            scheme: Scheme::UDP,
        }
    }
}

impl Url {
    pub fn new(base: &str) -> Self {
        let (scheme, base) = base.split_once(':').unwrap();
        let (base, port) = base.rsplit_once(':').expect("Invalid Address");
        let base = base.strip_prefix("//").unwrap();

        let port = if let Some((p, _)) = port.rsplit_once('/') {
            p
        } else {
            port
        };

        let port = port.parse::<u16>().unwrap();
        println!("{}, {}, {}", scheme, base, port);
        Self {
            url: (base, port)
                .to_socket_addrs()
                .expect("Failed to create socket")
                .next()
                .expect("No socket"),
            scheme: scheme.into(),
        }
    }
    pub fn port(&self) -> u16 {
        self.url.port()
    }
    pub fn addr(&self) -> IpAddr {
        self.url.ip()
    }
}
