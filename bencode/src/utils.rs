pub fn bcode_to_u8<'a>(bcode: &'a str) -> impl Iterator<Item = u8> + 'a {
    bcode.bytes().map(|b| b)
}
