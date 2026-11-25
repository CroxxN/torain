use crate::{
    error::DHTError,
    serde::{MessageTag, MessageType, NodeInfo, QueryType, ResponseType, KRPC},
};
use crypto::sha1;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time,
};
use tokio::{sync::Mutex, task::JoinSet};
use uttd::{
    url::{Scheme::UDP, Url},
    AsyncStream,
};

// we need a few nodes to "bootstrap" our DHT table.
// pub async fn bootstrap() -> Result<(), UttdError> {
// List of other bootstrap nodes:
//
// 1. "router.bittorrent.com:6881" (alias to 2.), *PREFFERED*
// 2. "router.utorrent.com:6881",
// 3. "router.bitcomet.com:6881",
// 4. "dht.transmissionbt.com:6881",
// 5. "dht.aelitis.com:6881",

// use "udp" here, despite not requiring the scheme, to let
// our url library know that we want to create a *UDP* async stream.

const K: u8 = 8;
const ALPHA: u8 = 3;

#[derive(Debug, PartialEq)]
pub enum PrefferedNetwork {
    UTORRENT,
    Bitorrent,
    Transmission,
    Aelitis,
}

pub fn xor_distance(node: &[u8; 20], target: &[u8; 20]) -> [u8; 20] {
    let mut res = [0u8; 20];
    node.iter().zip(target).enumerate().for_each(|(i, (x, y))| {
        res[i] = x ^ y;
    });
    res
}

#[derive(Debug)]
pub struct DHT {
    node_id: [u8; 20],
    in_flight: HashMap<[u8; 2], MessageTag>, // maybe HashMap<[u8; 2], &'a KRPC>
    table: Mutex<RouteTable>,
}

impl DHT {
    fn new() -> Result<Self, DHTError> {
        // Generate node_id using two phases
        // BEP_0005 recommends entropy for node id generation which is
        // achieved here by two phase involving current duration in seconds from
        // standard unix time.
        // The duration is multiplied by pseudo-random TinyMT number using the duration
        // as the seed.
        // The result is used to generation a sha1 output, which is the final node_id.
        let mut sha1_gen = sha1::Sha1::new();

        let seed = time::SystemTime::now().duration_since(time::SystemTime::UNIX_EPOCH)?;
        let seed = seed.as_secs() as u32;

        let seed = (crypto::tinymt::TinyMT::rand(seed).get_u32() as u64) * (seed as u64);

        sha1_gen.append_hash(&seed.to_be_bytes());

        let node_id = sha1_gen.get_hash();

        Ok(Self {
            node_id,
            in_flight: HashMap::new(),
            table: Mutex::new(RouteTable::new()),
        })
    }
}

// Routing table
#[derive(Debug, PartialEq)]
pub struct RouteTable {
    n: u16, // number of k buckets rn
    // paramet k = 8, for each i, 0<i<160, store a k-bucket, each bucket with a maximum of k nodes
    buckets: Vec<Vec<NodeInfo>>,
}

// TODO: Implement the routing table
impl RouteTable {
    pub fn new() -> Self {
        Self {
            n: 0,
            buckets: vec![Vec::new()],
        }
    }
    // tree update algorithm
    pub fn update(&mut self, info: Vec<NodeInfo>) {
        self.buckets.push(info);
        // if info.len() - (self.n as usize) <= 8 {
        //     self.buckets.push(info);
        // } else {
        //     todo!()
        // }
    }
    // slit vector. for example if the bucket is empty with len() = 0, adding a node doesn't need resize
    // if len()>8, 1 bucket is filled so we need to slit the bucket into 2. with the nodes uniformly divided between the
    // two buckets.
    fn resize_bucket(&mut self) {
        todo!()
    }

    pub fn get_kclosest(&mut self) {
        todo!()
    }
}

impl DHT {
    fn new_nodeid() -> [u8; 20] {
        let mut sha1_gen = sha1::Sha1::new();

        let seed = time::SystemTime::now()
            .duration_since(time::SystemTime::UNIX_EPOCH)
            .expect("System time configured wrong");
        let seed = seed.as_secs() as u32;

        let seed = (crypto::tinymt::TinyMT::rand(seed).get_u32() as u64) * (seed as u64);

        sha1_gen.append_hash(&seed.to_be_bytes());

        let id = sha1_gen.get_hash();

        id
    }

    pub fn form_tt() -> [u8; 2] {
        // TODO: for actual tt
        DHT::new_nodeid()[0..2].try_into().unwrap()
    }
    pub async fn bootstrap(network: PrefferedNetwork) -> Result<Arc<Self>, DHTError> {
        // Use a prefrerred network, if ping fails, try other
        let base = match network {
            PrefferedNetwork::UTORRENT => "router.utorrent.com",
            PrefferedNetwork::Transmission => "dht.transmissionbt.com",
            PrefferedNetwork::Aelitis => "dht.aelitis.com",
            _ => "router.bittorrent.com",
        };

        let node_id = Self::new_nodeid();
        // let message = MessageType::Query {
        //     id: node_id,
        //     query: QueryType::Ping,
        // };

        // let transact_id = Self::form_tt();

        // let krpc = KRPC::new(transact_id.clone(), message);

        let mut base = Url::from_ip(base, 6881)?;
        base.scheme = UDP;
        // let mut udp = AsyncStream::new(&base).await.unwrap();

        // let mut response = vec![0u8; 100];

        // let size = udp
        //     .send(krpc.serialize()?.as_slice(), &mut response)
        //     .await?; // TODO: remove unwrap
        // response.truncate(size);
        // // don't need tt because we already know the message hint + we only sent one KRPC message
        // let (_, btrees) = KRPC::get_tt(response)?;

        // let krpc_response = KRPC::convert(btrees, Some(MessageTag::Ping))?;
        // WIP: assuming ping succeeds
        // if let MessageType::Response { id, response } = krpc_response.message_type{
        // }

        // For bootstrap, we need to know one node already in the network. This work is done by the bootstrap nodes in
        // bittorrent. We first ping the bootstrap node to get the node id. We need to PING the bootstrap node's IP first
        // to get the node id. We insert the received node id into the appropiate k-bucket. Then we perform a `find_node`
        // on our own node id. We get a bunch of nodes closer to our own from the bootstrap node (potentially). We then
        // repeatedly query the returned nodes to find nodes closer to ours and add the nodes to our k-bucket. Then, we have
        // joined the network.

        let dht = Arc::new(Self {
            node_id,
            in_flight: HashMap::new(),
            table: Mutex::new(RouteTable::new()),
        });
        // we don't care about the result here because the return value is our own node_id
        // for bootstrap
        let _ = dht.find_node(base, node_id).await?;
        Ok(dht)
    }

    async fn find_node(
        self: &Arc<Self>,
        url: Url,
        id: [u8; 20],
    ) -> Result<Vec<NodeInfo>, DHTError> {
        let mut node_queue: HashSet<[u8; 20]> = HashSet::new();
        // round one
        let mut nodes = self.distributed_find(id, url).await?;

        let mut jset = JoinSet::new();

        loop {
            nodes.sort_by_key(|n| xor_distance(&n.id, &id));
            // alpha parameter
            for n in nodes.iter().take(K.into()) {
                if jset.len() >= ALPHA as usize {
                    break;
                }
                if node_queue.insert(n.id) {
                    let rt = Arc::clone(self);
                    let url = n.url.clone();
                    jset.spawn(async move { rt.distributed_find(id, url).await });
                }
            }
            let Some(res) = jset.join_next().await else {
                break;
            };

            let Ok(Ok(res)) = res else {
                continue;
            };

            self.table.lock().await.update(nodes.clone());

            for n in res {
                if n.id == self.node_id {
                    continue;
                }
                if n.id == id {
                    jset.abort_all();
                    // update table
                    return Ok(vec![n]);
                }
                if !nodes.iter().any(|m| m.id == n.id) {
                    nodes.push(n);
                }
            }
            // do something
        }

        nodes.sort_by_cached_key(|a| xor_distance(&a.id, &self.node_id));
        self.table.lock().await.update(nodes.clone());
        // if we didn't find the exact node, just return none. It does the job of going
        // through the nodes recursively.
        nodes.truncate(8);
        return Ok(nodes);
    }

    async fn distributed_find(&self, id: [u8; 20], url: Url) -> Result<Vec<NodeInfo>, DHTError> {
        let transact_id = Self::form_tt();

        let node_id = self.node_id;

        let query = MessageType::Query {
            id: node_id,
            query: QueryType::FindNode { target: id },
        };

        let krpc = KRPC::new(transact_id, query).serialize()?;
        let response = krpc.as_slice();

        let mut udp_response = vec![0u8; 1000];

        let mut stream = AsyncStream::new(&url).await?;
        let _ = stream.send(response, &mut udp_response).await?;

        // don't need tt because we already know the message hint
        let (_, btrees) = KRPC::get_tt(udp_response)?;
        let krpc_response = KRPC::convert(btrees, Some(MessageTag::FindNode))?;

        match krpc_response.message_type {
            MessageType::Response {
                response: ResponseType::FindNode { nodes },
                ..
            } => Ok(nodes),
            MessageType::Error(e) => Err(e),
            _ => Err(DHTError::ServerError),
        }
    }

    async fn find_peer(&mut self, info_hash: [u8; 20]) {
        let _ = info_hash;
        todo!()
    }
}

#[cfg(test)]
mod test {
    use tokio::test;

    use crate::{error::DHTError, kademlia::DHT};

    #[test]
    async fn bootstrap() -> Result<(), DHTError> {
        let dht = DHT::bootstrap(crate::kademlia::PrefferedNetwork::Transmission).await?;
        let tables = &dht.table.lock().await.buckets;
        dbg!(tables.len());
        let len: usize = tables
            .iter()
            .enumerate()
            .map(|(idx, x)| {
                print!("{} idx, len: {}\n", idx, x.len());
                x.len()
            })
            .sum();
        dbg!(len);
        assert!(!tables.is_empty());
        Ok(())
    }
}
