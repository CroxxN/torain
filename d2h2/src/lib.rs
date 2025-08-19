// REFERENCE: https://www.bittorrent.org/beps/bep_0005.html

// TODO:
// = implement a routing table
// = implement the XOR metric
// = more

use std::sync::Arc;

use error::D2H2ClientError;
use uttd::url::Url;

mod error;
mod kademlia;
mod request;
mod serde;

pub enum PrefferedNetwork {
    UTORRENT,
    Bitorrent,
    Transmission,
    Aelitis,
}

#[derive(Debug)]
#[allow(dead_code)]
struct D2H2Client {
    // TODO: Complete
    // URL Paths that should be contacted to find peers
    // this vec is updated everytime "k-closest" node is returned from
    // a DHT node instead of peers
    // MAYBE: add an arc here
    paths: Vec<Url>,
    // number of concurrent requests to send for path
    concurrent: u8,
}

impl D2H2Client {
    pub fn new(preffered: Option<PrefferedNetwork>) -> Result<Self, D2H2ClientError> {
        let base = match preffered {
            Some(PrefferedNetwork::UTORRENT) => "router.utorrent.com",
            Some(PrefferedNetwork::Transmission) => "dht.transmissionbt.com",
            Some(PrefferedNetwork::Aelitis) => "dht.aelitis.com",
            _ => "router.bittorrent.com",
        };
        let base = Url::from_ip(base, 6881)?;
        Ok(Self {
            paths: vec![base],
            concurrent: 5, // TODO: change this value
        })
    }
    pub fn find_peers(&self, info_hash: &[u8; 20]) -> Result<Box<[Url]>, D2H2ClientError> {
        let urls: Arc<Vec<Url>> = Arc::new(Vec::new());
        // TODO:
        // 1. make requests self.paths
        // 2. if returend nodes, add to urls
        match Arc::try_unwrap(urls) {
            Ok(v) => Ok(v.into_boxed_slice()),
            Err(_) => Err(D2H2ClientError::MoveOutofArcError),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::D2H2Client;

    #[test]
    fn find_peers_dht() {
        let client = D2H2Client::new(None).unwrap();
        // TODO: implement this test
        _ = client.find_peers(&[0; 20]);
    }
}
