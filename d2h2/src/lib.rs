#[derive(Debug)]
pub struct Node {
    id: [u8; 20],
    routing: RTable,
}

// Routing table
#[derive(Debug)]
pub struct RTable {}

fn bootstrap() {
    let bootstraps = [
        "router.bittorrent.com:6881",
        "router.utorrent.com:6881",
        "router.bitcomet.com:6881",
        "dht.transmissionbt.com:6881",
        "dht.aelitis.com:6881",
    ];
}

#[cfg(test)]
mod tests {}
