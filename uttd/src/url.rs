use crate::error::UrlError;

/// Url
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Url {
    /// Scheme of the url --- HTTPS, HTTP or UDP
    pub scheme: Scheme,
    /// Host of the server --- in the form of {domain}:{port}
    pub host: String,
    /// additional path --- default to '/'
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
            scheme: Scheme::UDP,
            host: "127.0.0.1".to_owned(),
            location: "/".to_owned(),
        }
    }
}

impl<'a> Url {
    /// Create a new 'Url' from a base address
    /// The `address` must be in the form "<scheme://address:port/{path...}>"

    /// ```
    /// use uttd::url::Url;
    /// let url = Url::new("http://google.com:80/some_page").unwrap();
    /// ```

    pub fn new(address: &'a str) -> Result<Self, UrlError> {
        let (scheme, base) = address.split_once(':').ok_or(UrlError::InvalidUrl)?;
        let mut base = base.strip_prefix("//").ok_or(UrlError::InvalidUrl)?;

        // let (base, port) = base.rsplit_once(':').ok_or(UrlError::InvalidUrl)?;
        let mut loc = "/";

        if let Some((b, location)) = base.rsplit_once('/') {
            base = b;
            loc = location;
        }

        Ok(Self {
            scheme: scheme.into(),
            host: base.to_owned(),
            location: loc.to_owned(),
        })
    }
    /// Create a `Url` from bytes in form of [x, x, x, x]
    /// IPv4
    pub fn from_ip_bytes(ip: &'a [u8], port: u16) -> Self {
        let mut ip_addr = String::new();
        ip.iter().for_each(|x| ip_addr.push_str(&format!("{}.", x)));
        ip_addr.pop();
        let ip = format!("{}:{}", ip_addr, port);
        Self {
            scheme: Scheme::HTTP,
            host: ip,
            location: "/".to_owned(),
        }
    }

    /// Create `Url` from a string of ip address
    pub fn from_ip(ip: &'a str, port: u16) -> Result<Self, UrlError> {
        let ip = format!("{}:{}", ip, port);
        Ok(Self {
            scheme: Scheme::HTTP,
            host: ip,
            location: "/".to_owned(),
        })
    }

    /// Get the port associated with the remote address
    /// ```
    /// use uttd::url::Url;
    /// let url = Url::new("http://google.com:80/some_page").unwrap();
    /// assert_eq!(80, url.port());
    /// ```

    pub fn port(&self) -> u16 {
        let (_, port) = self.host.split_once(':').unwrap();
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

        assert_eq!(
            (port, scheme, host),
            (6969, Scheme::HTTP, "bttracker.debian.org:6969".to_owned())
        );
    }

    #[test]
    fn udp_announce() {
        let url = Url::new("udp://open.demonii.com:1337").unwrap();

        let port = url.port();
        let scheme = url.scheme;
        let host = url.host;

        assert_eq!(
            (port, scheme, host),
            (1337, Scheme::UDP, "open.demonii.com:1337".to_owned())
        );
    }
}
