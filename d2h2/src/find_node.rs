use uttd::url::Url;

pub enum PrefferedNetwork {
    UTORRENT,
    Bitorrent,
    Transmission,
    Aelitis,
}

#[derive(Debug)]
pub struct NodeFinder {
    base: Url,
}

impl NodeFinder {
    pub fn new(preffered: Option<PrefferedNetwork>) -> Self {
        let base = match preffered {
            Some(PrefferedNetwork::UTORRENT) => "router.utorrent.com",
            Some(PrefferedNetwork::Transmission) => "dht.transmissionbt.com",
            Some(PrefferedNetwork::Aelitis) => "dht.aelitis.com",
            _ => "router.bittorrent.com",
        };
        // TODO: fix this unwarp
        let base = Url::from_ip(base, 6881).unwrap();
        Self { base }
    }
    pub fn get_peers(&self, hash: [u8; 20]) {
        // TODO: implement
        // 1. Make a initial request to the base bootstrapper
        // 2. Get a list of node from the base
        // 3. Make a get_peer request to the node
        // 4. if found peers, continue
        // 5. if not, make the newly retured ip addresses the base and follow step 2
        unimplemented!()
    }
}

#[cfg(test)]
mod test {

    use crate::find_node::NodeFinder;

    #[test]
    fn get_peers() {
        let torrent_hash = [0u8; 20];
        let nf = NodeFinder::new(None);
        let peers = nf.get_peers(torrent_hash);
        // assert!(!peers.is_empty());
    }
}
