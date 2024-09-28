pub mod bencode;
pub mod utils;
use bencode::decode;
use utils::bcode_to_u8;

// IMPORTANT: Uncomment this line next

// struct Torrent {
//     // define the fields that are generally used in torrent files. Things like trackers, filenames, etc.
// }

fn main() {
    let data = "li-556e4:highi34eeli45e3:broe";
    let mut u8s = bcode_to_u8(data);
    decode(&mut u8s);
    // println!("{}", decode(&mut u8s).expect("Failed"));
}

// fn init_decode(data: &str) {
//     let item = data.chars().map(|c| c);
// }
