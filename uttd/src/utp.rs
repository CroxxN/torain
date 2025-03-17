use std::time::{SystemTime, UNIX_EPOCH};

// use tokio::net::UdpSocket;

// use crate::url::Url;

const ST_DATA: u8 = 0;
const ST_FIN: u8 = 1;
const ST_STATE: u8 = 2;
const ST_RESET: u8 = 3;
const ST_SYN: u8 = 4;

/// Async uTorrent Transport Protocol

// 0       4       8               16              24              32
// +-------+-------+---------------+---------------+---------------+
// | type  | ver   | extension     | connection_id                 |
// +-------+-------+---------------+---------------+---------------+
// | timestamp_microseconds                                        |
// +---------------+---------------+---------------+---------------+
// | timestamp_difference_microseconds                             |
// +---------------+---------------+---------------+---------------+
// | wnd_size                                                      |
// +---------------+---------------+---------------+---------------+
// | seq_nr                        | ack_nr                        |
// +---------------+---------------+---------------+---------------+
#[derive(Eq, PartialEq, PartialOrd, Ord)]
// #[repr(C, packed)]
pub struct UtpPacket {
    packet_version: u8,
    extension: u8,
    connection_id: u16,
    timestamp: u32,
    timestamp_difference: u32,
    window_size: u32,
    /// sequence number
    seq_number: u16,
    /// acknowledged number
    ack_number: u16,
}

impl UtpPacket {
    pub fn new() -> Self {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        let timestamp = (timestamp.as_secs().wrapping_add(1_000_000) as u32)
            .wrapping_add(timestamp.subsec_micros())
            .into();

        Self {
            // first message is the ST_SYN message
            // the first four bits from the left is the version number (always one)
            // and the last 4 are packet id
            packet_version: (ST_SYN << 4) | 1,
            extension: 0,
            connection_id: 0x35,
            timestamp,
            timestamp_difference: 0,
            window_size: 0xf000,
            // initialized to 1
            seq_number: 1,
            ack_number: 0,
        }
    }

    pub fn refetch_timestamp(&mut self) {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let timestamp = (timestamp.as_secs().wrapping_add(1_000_000) as u32)
            .wrapping_add(timestamp.subsec_micros())
            .into();

        self.timestamp = timestamp;
    }

    pub fn as_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.packet_version.to_be_bytes());
        bytes.extend_from_slice(&self.extension.to_be_bytes());
        bytes.extend_from_slice(&self.connection_id.to_be_bytes());
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());
        bytes.extend_from_slice(&self.timestamp_difference.to_be_bytes());
        bytes.extend_from_slice(&self.window_size.to_be_bytes());
        bytes.extend_from_slice(&self.seq_number.to_be_bytes());
        bytes.extend_from_slice(&self.ack_number.to_be_bytes());
        // println!("{}", bytes.len());
        bytes
    }
}

// pub struct UtpSocket {
//     pub socket: UdpSocket,
// }

// impl UtpSocket {
//     pub async fn new(url: &Url) -> Self {
//         let sock = UdpSocket::bind("0.0.0.0:0").await.unwrap();
//         sock.connect(&url.host).await.unwrap();

//         Self { socket: sock }
//     }
// }

#[cfg(test)]
mod test {
    // use crate::utp::ST_SYN;

    use super::UtpPacket;

    #[test]
    fn new_packet() {
        let upacket = UtpPacket::new();
        assert_eq!(upacket.packet_version, 65);
    }
}
