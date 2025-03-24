#![allow(dead_code)]

mod parse;
mod utils;

use uttd::{url::Url, AsyncStream};

// which type of query is this packet?
enum Query {
    PING,
    FindNode,
    GetPeers,
    AnnouncePeer,
}

#[derive(Debug)]
pub struct DHT {
    node: Node,
    table: RTable,
}

impl DHT {
    pub async fn new() -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub struct Node {
    id: [u8; 20],
    routing: RTable,
}

impl Node {}

// Routing table
#[derive(Debug)]
pub struct RTable {}

// we need a few nodes to "bootstrap" our DHT table.
// pub async fn bootstrap() -> Result<(), UttdError> {
pub async fn bootstrap() -> AsyncStream {
    // TODO: use these all
    // let bootstraps = [
    //     "router.bittorrent.com:6881",
    //     "router.utorrent.com:6881",
    //     "router.bitcomet.com:6881",
    //     "dht.transmissionbt.com:6881",
    //     "dht.aelitis.com:6881",
    // ];
    // we use "udp" here to let our url library know that we want to create a *UDP* async stream
    let bootstrap = Url::new("udp://router.bittorrent.com:6881").unwrap();
    let stream = AsyncStream::new(bootstrap).await.unwrap();
    stream
}

// WORKS
// TODO: Create a nice way to interact with the DHT server.
#[cfg(test)]
mod tests {

    use bencode::{bencode::BTypes, bencoen};

    use crate::bootstrap;

    #[tokio::test]
    async fn connect_dht() {
        let mut s = bootstrap().await;
        println!("{:?}", s);

        let id = bencoen::Bencoen::new(
            "id".to_string(),
            // TODO: Change this to reflect actual node id. maybe generate using sha
            BTypes::BSTRING("abcdefghij0123456789".into()),
        );

        let mut dict = bencoen::Bencoen::new("t".to_string(), BTypes::BSTRING("aa".into()));

        dict.add("y".to_string(), BTypes::BSTRING("q".into()));
        dict.add("q".to_string(), BTypes::BSTRING("ping".into()));

        dict.add("a".to_string(), id.get_inner());

        let encoded = dict.finalize();

        let mut res = vec![0; 100];
        _ = s.send(&encoded, &mut res).await.unwrap();

        let decoded =
            bencode::bencode::decode(&mut res.into_iter()).expect("Unable to decode bytes");

        if let BTypes::DICT(d) = decoded {
            if let Some(_) = d.get("y") {
                return ();
            } else {
                panic!("Key 'y' not present.");
            }
        }
        panic!("No Dictionary present.")
    }
}
