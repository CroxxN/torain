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
    /// Create a new 'Url' from a base address
    /// The `address` must be in the form "<scheme://address:port/{path...}>"

    /// ```
    /// use uttd::url::Url;
    /// let url = Url::new("http://google.com:80/some_page").unwrap();
    /// ```

    pub fn new(address: &str) -> Result<Self, UrlError> {
        let (scheme, base) = address.split_once(':').ok_or(UrlError::InvalidUrl)?;
        let mut base = base.strip_prefix("//").ok_or(UrlError::InvalidUrl)?;
        // let (base, port) = base.rsplit_once(':').ok_or(UrlError::InvalidUrl)?;

        let mut location = "/".to_owned();

        if let Some((b, loc)) = base.rsplit_once('/') {
            base = b;
            location = format!("/{}", loc);
        }

        Ok(Self {
            url: base.to_owned(),
            scheme: scheme.into(),
            host: base.to_owned(),
            location,
        })
    }
    pub fn from_ip_bytes(ip: [u8; 4], port: u16) -> Self {
        let mut ip_addr = String::new();
        // let mut ip = ip.iter().map(|x| *x as char).collect::<String>();
        ip.iter()
            .for_each(|x| ip_addr.push_str(&format!("{}.", *x)));
        ip_addr.pop();
        ip_addr.push_str(&format!(":{}", port));
        Self {
            url: ip_addr.clone(),
            scheme: Scheme::HTTP,
            host: ip_addr,
            location: "/".to_string(),
        }
    }

    pub fn from_ip(ip: &str, port: u16) -> Result<Self, UrlError> {
        let ip_address = format!("{}:{}", ip, port);
        Ok(Self {
            url: ip_address.clone(),
            scheme: Scheme::HTTP,
            host: ip_address,
            location: "/".to_string(),
        })
    }

    /// Get the port associated with the remote address
    /// ```
    /// use uttd::url::Url;
    /// let url = Url::new("http://google.com:80/some_page").unwrap();
    /// assert_eq!(80, url.port());
    /// ```

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
                "/announce".to_owned()
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
