// #[allow(unused_variables, dead_code)]

pub fn encode(value: &str) -> String {
    let mut formatted = String::new();

    value.bytes().for_each(|x| {
        if x.is_ascii_alphanumeric() {
            formatted.push(x as char)
        } else {
            formatted.push_str(&format!("%{:X}", x))
        }
    });

    formatted
}

#[cfg(test)]
mod test {
    use super::encode;

    #[test]
    fn check_valid_ascii() {
        let value = "abc";
        assert_eq!(value, &encode(value));
    }

    #[test]
    fn check_formatted_bytes() {
        let value = "Û";
        assert_eq!("%C3%9B", &encode(value));
    }
}
