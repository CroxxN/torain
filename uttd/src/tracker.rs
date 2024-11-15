pub struct TrackerParams {
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
    pub port: [u8; 4],
    pub uploaded: [u8; 4],
    pub downloaded: [u8; 4],
    pub left: [u8; 4],
    pub compact: u8,
    pub event: Event,
    pub trackerid: Option<[u8; 20]>,
}

pub enum Event {
    Started,
    Stopped,
    Completed,
}

impl TrackerParams {
    fn new(info_hash: [u8; 20]) -> Self {
        unimplemented!()
    }
}
