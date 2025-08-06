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

    use crate::request::{Query, RequestBucket};

    fn check_bucket() {
        let transaction_id = "aa";

        let mut bucket = RequestBucket::new();
        bucket.add_request(transaction_id, Query::PING);

        let query = bucket.get_request(transaction_id);

        assert_eq!(query, Some(Query::PING));
    }
}
