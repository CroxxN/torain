use std::{collections::HashMap, str::FromStr};

// #[allow(unused_variables, dead_code)]
// use crate::tracker::TrackerParams;

fn encode(value: &[u8]) -> String {
    let mut formatted = String::new();

    value.iter().for_each(|x| match *x {
        b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'.' | b'-' | b'_' => formatted.push(*x as char),

        _ => formatted.push_str(&format!("%{:X}", x)),
    });

    formatted
}

pub fn build_url(base: &str, params: HashMap<&str, Vec<u8>>) -> String {
    let mut url: String = String::from_str(base).expect("FAILED to create String");

    url.push('?');

    for (k, v) in params {
        url.push_str(k);
        url.push('=');
        url.push_str(&encode(v.as_slice()));
        url.push('&');
    }
    url.pop();

    url
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::urutil::build_url;

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

    // we only test with a single parameter as testing with multiple parameter is not possible.
    // Hashmap doesn't preserve order, but it should be good enought for urls as order doesn't matter
    // in urls anyway.
    #[test]
    fn check_url_param() {
        let base = "https://google.com";
        let mut params = HashMap::new();
        params.insert("cookie", "not available".to_owned().into_bytes());
        assert_eq!(
            "https%3A%2F%2Fgoogle.com?cookie=not%20available".to_owned(),
            build_url(base, params)
        );
    }
}
