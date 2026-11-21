#![allow(unused_imports, dead_code)]

use std::collections::HashMap;

use crate::serde;

// which type of query is this packet?
// TODO: for now, we're only concerned with sending
// FindNode queries from OUR side. We disregard all incoming requests for now.
#[derive(Debug, PartialEq)]
pub enum Query {
    PING,
    FindNode,
    GetPeers,
    AnnouncePeer,
}

// A bucket to store and hash queries with their associate transaction id.
pub struct RequestBucket<'a> {
    map: HashMap<&'a str, Query>,
}

impl<'a> RequestBucket<'a> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn add_request(&mut self, transaction_id: &'a str, query: Query) {
        self.map.insert(transaction_id, query);
    }
    pub fn get_request(&mut self, transaction_id: &'a str) -> Option<Query> {
        self.map.remove(transaction_id)
    }
}

#[cfg(test)]
mod tests {

    use uttd::url::Url;
    use uttd::Stream;

    use crate::request::{Query, RequestBucket};
    use crate::serde::*;

    #[test]
    fn check_bucket() {
        let transaction_id = "aa";

        let mut bucket = RequestBucket::new();
        bucket.add_request(transaction_id, Query::PING);

        let query = bucket.get_request(transaction_id);

        assert_eq!(query, Some(Query::PING));
    }
    #[test]
    fn ping() {
        let message = MessageType::Query {
            id: [0; 20],
            query: crate::serde::QueryType::Ping,
        };
        let krpc = KRPC::new(['a' as u8, 'a' as u8], message);
        let krpc_bytes = krpc.clone().serialize();
        krpc_bytes.iter().for_each(|x| print!("{}", *x as char));
        // dbg!(&krpc_bytes);
        let mut stream =
            Stream::new(&Url::new("udp://dht.transmissionbt.com:6881").unwrap()).unwrap();
        let mut resp = vec![0; 100];
        _ = stream.send(&krpc_bytes, &mut resp);
        // let resp = crate::serde::Vec::deserialize_bytes(resp);
        let resp: KRPC = crate::serde::Vec2Kprc::into(resp, Some(MessageTag::Ping)).unwrap();
        assert_eq!(resp.transact_id, [b'a', b'a']);
    }
    #[test]
    fn find_peer() {
        let fs = "debian.torrent";
        let torrent = torrent::torrent::Torrent::from_file(fs).unwrap();
        let tracker = torrent::tracker::TrackerParams::new(&torrent);
        let info_hash = tracker.info_hash.to_owned().try_into().unwrap();

        let transaction_id = [b'a', b'a'];
        let message = MessageType::Query {
            id: *b"abcdefghij0123456789",
            query: QueryType::GetPeers { info_hash },
        };

        let krpc = KRPC::new(transaction_id, message);

        let krpc_bytes = krpc.serialize();

        let mut stream =
            Stream::new(&Url::new("udp://dht.transmissionbt.com:6881").unwrap()).unwrap();
        //IMPORTANT: We need large vector because nodes have a habit of returning huge number of other nodes
        let mut resp = vec![0; 1000];
        _ = stream.send(&krpc_bytes, &mut resp);

        resp.iter().for_each(|x| print!("{}", *x as char));

        let resp = <KRPC as Vec2Kprc>::into(resp, Some(MessageTag::GetPeers));

        assert_eq!(resp.unwrap().transact_id, [b'a', b'a']);
    }
}
