/// Driver code for the dht engine

pub enum DHTCommands {
    Init,
    GetPeers([u8; 20]),
    Stop,
}

impl Default for DHTCommands {
    fn default() -> Self {
        return Self::Init;
    }
}
