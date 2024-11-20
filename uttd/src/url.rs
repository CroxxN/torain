use crate::error::UrlError;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Url<'a> {
    pub url: &'a str,
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
            url: "127.0.0.1:0",
            scheme: Scheme::UDP,
            host: "127.0.0.1",
            location: "/",
        }
    }
}

impl<'a> Url<'a> {
    pub fn new(address: &'a str) -> Result<Self, UrlError> {
        let (scheme, base) = address.split_once(':').ok_or(UrlError::InvalidUrl)?;
        let mut base = base.strip_prefix("//").ok_or(UrlError::InvalidUrl)?;
        // let (base, port) = base.rsplit_once(':').ok_or(UrlError::InvalidUrl)?;

        let mut location = "/";

        if let Some((b, loc)) = base.rsplit_once('/') {
            base = b;
            location = loc;
        }
        println!("{}", base);

        Ok(Self {
            url: base,
            scheme: scheme.into(),
            host: base,
            location,
        })
    }

    pub fn port(&self) -> u16 {
        let (_, port) = self.url.split_once(':').unwrap();
        port.parse().unwrap()
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
            (6969, Scheme::HTTP, "bttracker.debian.org:6969", "announce")
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
            (1337, Scheme::UDP, "open.demonii.com:1337", "/")
        );
    }
}
