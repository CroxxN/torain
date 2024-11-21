use crate::error::UrlError;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Url {
    pub url: String,
    pub scheme: Scheme,
    pub host: String,
    pub location: String,
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

impl Default for Url {
    fn default() -> Self {
        Self {
            url: "127.0.0.1:0".to_owned(),
            scheme: Scheme::UDP,
            host: "127.0.0.1".to_owned(),
            location: "/".to_owned(),
        }
    }
}

impl Url {
    pub fn new(address: &str) -> Result<Self, UrlError> {
        let (scheme, base) = address.split_once(':').ok_or(UrlError::InvalidUrl)?;
        let mut base = base.strip_prefix("//").ok_or(UrlError::InvalidUrl)?;
        // let (base, port) = base.rsplit_once(':').ok_or(UrlError::InvalidUrl)?;

        let mut location = "/";

        if let Some((b, loc)) = base.rsplit_once('/') {
            base = b;
            location = loc;
        }

        Ok(Self {
            url: base.to_owned(),
            scheme: scheme.into(),
            host: base.to_owned(),
            location: location.to_owned(),
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
            (
                6969,
                Scheme::HTTP,
                "bttracker.debian.org:6969".to_owned(),
                "announce".to_owned()
            )
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
            (
                1337,
                Scheme::UDP,
                "open.demonii.com:1337".to_owned(),
                "/".to_owned()
            )
        );
    }
}
