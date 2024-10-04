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
    // let data = "3:h\nw";
    // let mut fd = fs::File::open("pulp_fiction.torrent").expect("Failed to open file");
    // let mut data = String::new();
    // fd.read_to_string(&mut data).expect("Failed to open file");
    let mut raw =
        fs::read("debian-12.7.0-amd64-netinst.iso.torrent").expect("Failed to parse file");
    _ = raw.pop();
    let mut u8s = raw.into_iter();
    // let mut u8s = bcode_to_u8(data);
    decode(&mut u8s);
    // println!("{}", decode(&mut u8s).expect("Failed"));
}

// fn init_decode(data: &str) {
//     let item = data.chars().map(|c| c);
// }
