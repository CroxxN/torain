// #[allow(unused_variables, dead_code)]

pub fn encode(value: &[u8]) -> String {
    let mut formatted = String::new();

    value.iter().for_each(|x| match *x {
        b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'.' | b'-' | b'_' => formatted.push(*x as char),

        _ => formatted.push_str(&format!("%{:X}", x)),
    });

    formatted
}

pub fn build_url(base: &str) {
    let mut url_space: Vec<u8> = base.as_bytes().iter().map(|x| *x).collect();
    url_space.push(b'?');
    unimplemented!()
}

#[cfg(test)]
mod test {
    use super::encode;

    #[test]
    fn check_valid_ascii() {
        let value = "abc";
        assert_eq!(value, &encode(value.as_bytes()));
    }

    #[test]
    fn check_formatted_bytes() {
        let value = "Û";
        assert_eq!("%C3%9B", &encode(value.as_bytes()));
    }
    #[test]
    fn check_space() {
        let value = " ";
        assert_eq!("%20", &encode(value.as_bytes()));
    }
    #[test]
    fn check_url_basic() {
        let value = "https://google.com/";
        assert_eq!("https%3A%2F%2Fgoogle.com%2F", &encode(value.as_bytes()));
    }
    #[test]
    fn check_url_params() {
        let value = "https://google.com?cookie=not available";
        assert_eq!(
            "https%3A%2F%2Fgoogle.com%3Fcookie%3Dnot%20available",
            &encode(value.as_bytes())
        );
    }
}
