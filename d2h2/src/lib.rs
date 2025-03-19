use uttd::{url::Url, AsyncStream};

#[derive(Debug)]
pub struct Node {
    id: [u8; 20],
    routing: RTable,
}

// Routing table
#[derive(Debug)]
pub struct RTable {}

impl Node {}

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
    use std::collections::BTreeMap;

    use bencode::bencode::BTypes;

    use crate::bootstrap;

    #[tokio::test]
    async fn connect() {
        let mut s = bootstrap().await;
        println!("{:?}", s);

        let mut id: BTreeMap<String, BTypes> = BTreeMap::new();
        id.insert(
            "id".to_string(),
            BTypes::BSTRING("abcdefghij0123456789".into()),
        );

        let mut dict: BTreeMap<String, BTypes> = BTreeMap::new();

        dict.insert("t".to_string(), BTypes::BSTRING("aa".into()));
        dict.insert("y".to_string(), BTypes::BSTRING("q".into()));
        dict.insert("q".to_string(), BTypes::BSTRING("ping".into()));

        dict.insert("a".to_string(), BTypes::DICT(id));

        let bcon = BTypes::DICT(dict);

        println!("{:?}", bcon);

        let encoded = bencode::bencoen::ser(&bcon);
        println!("{:?}", encoded);

        let mut res = vec![0; 100];

        let read = s.send(&encoded, &mut res).await.unwrap();
        println!("Read: {read}");

        let decoded =
            bencode::bencode::decode(&mut res.into_iter()).expect("Unable to decode bytes");
        println!("Decoded: {:?}", decoded);
        // outer["t"] = BTypes::BSTRING("aa".into());
    }
}
