pub mod bencode;
pub mod utils;
use bencode::decode;
use std::fs;

// IMPORTANT: Uncomment this line next

// struct Torrent {
//     // define the fields that are generally used in torrent files. Things like trackers, filenames, etc.
// }

fn main() {
    // test data
    let _data = "d8:announce27:udp://open.demonii.com:1337";
    // let mut fd = fs::File::open("pulp_fiction.torrent").expect("Failed to open file");
    // let mut data = String::new();
    // fd.read_to_string(&mut data).expect("Failed to open file");
    let mut u8s = fs::read("pulp_fiction.torrent")
        .expect("Failed to parse file")
        .into_iter();
    decode(&mut u8s);
    // println!("{}", decode(&mut u8s).expect("Failed"));
}

// fn init_decode(data: &str) {
//     let item = data.chars().map(|c| c);
// }
