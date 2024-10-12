struct Torrent {
    announce: String,
    announce_list: Vec<String>,
    info: Info,
}

struct Info {
    name: String,
    piece_length: u64,
    pieces: Vec<u8>,
}
