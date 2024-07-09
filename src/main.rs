pub mod bencode;
use bencode::decode;

fn main() {
    let data = "i42e";
    decode(data);
}
