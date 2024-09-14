pub mod bencode;
use bencode::bcode_to_u8;

// IMPORTANT: Uncomment this line next

// struct Torrent {
//     // define the fields that are generally used in torrent files. Things like trackers, filenames, etc.
// }

fn main() {
    let data = "i-0e";
    println!("{}", bcode_to_u8(data));
}

// fn init_decode(data: &str) {
//     let item = data.chars().map(|c| c);
// }
