
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::net::UdpSocket;

use crate::url::Url;

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
#[repr(C, packed)]
pub struct UtpPacket {
    packet_type: u8,
    extension: u8,
    connection_id: i16,
    timestamp: i32,
    timestamp_difference: i32,
    window_size: i32,
    /// sequence number
    seq_number: i16,
    /// acknowledged number
    ack_number: i16,
}

impl UtpPacket {
    fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i32;

        Self {
            packet_type: ST_SYN,
            extension: 0,
            connection_id: 0,
            timestamp,
            timestamp_difference: 0,
            window_size: 0,
            seq_number: 1,
            ack_number: 0,
        }
    }

    fn refetch_timestamp(&mut self) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i32;

        self.timestamp = timestamp;
    }

    fn as_bytes(self) -> Vec<u8> {
        let bytes = unsafe {
            std::slice::from_raw_parts(
                &self as *const Self as *mut Self as *mut u8,
                std::mem::size_of::<Self>(),
            )
        };
        bytes.to_vec()
    }
}

pub struct UtpSocket {
    pub socket: UdpSocket,
}

impl UtpSocket {
    pub async fn new(url: &Url) -> Self {
        let sock = UdpSocket::bind("0.0.0.0:0").await.unwrap();
        sock.connect(&url.host).await.unwrap();

        Self { socket: sock }
    }
}

#[cfg(test)]
mod test {
    use crate::utp::ST_SYN;

    use super::UtpPacket;

    #[test]
    fn new_packet() {
        let upacket = UtpPacket::new();
        assert_eq!(upacket.packet_type, ST_SYN);
    }
}
