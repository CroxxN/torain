pub fn bcode_to_u8<'a>(bcode: &'a str) -> impl Iterator<Item = u8> + 'a {
    let u8s = bcode.bytes().map(|b| b);
    u8s
}
