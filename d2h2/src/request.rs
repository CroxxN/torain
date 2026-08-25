use std::collections::HashMap;

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
        let transaction_id = "aa";
        let krpc = KRPC::new(
            transaction_id.into(),
            QueryType::Ping,
            b"abcdefghij0123456789",
        );
        let krpc_bytes = serialize(krpc);
        krpc_bytes.iter().for_each(|x| print!("{}", *x as char));
        // dbg!(&krpc_bytes);
        let mut stream =
            Stream::new(&Url::new("udp://dht.transmissionbt.com:6881").unwrap()).unwrap();
        let mut resp = vec![0; 100];
        let result = stream.send(&krpc_bytes, &mut resp);
        _ = dbg!(result);
        dbg!(&resp);
        println!("RESPONSE");
        resp.iter().for_each(|x| {
            if x.is_ascii() {
                print!("{}", *x as char)
            } else {
                print!("*");
            }
        });
    }
}
