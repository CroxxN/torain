use std::collections::HashMap;

// #[allow(unused_variables, dead_code)]
// use crate::tracker::TrackerParams;

fn encode(value: &[u8]) -> Vec<u8> {
    let mut formatted = vec![];

    value.iter().for_each(|x| match *x {
        b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'.' | b'-' | b'_' => formatted.push(*x),

        _ => formatted.extend(format!("%{:X}", x).as_bytes()),
    });

    formatted
}

pub fn build_url(base: &str, params: HashMap<&str, Vec<u8>>) -> Vec<u8> {
    let mut url: Vec<u8> = encode(base.as_bytes()).iter().map(|x| *x).collect();

    url.push(b'?');

    for (k, v) in params {
        url.extend(k.as_bytes());
        url.push(b'=');
        url.extend(encode(&v));
        url.push(b'&');
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
        assert_eq!(value.as_bytes(), &encode(value.as_bytes()));
    }

    #[test]
    fn check_formatted_bytes() {
        let value = "Û";
        assert_eq!("%C3%9B".as_bytes(), &encode(value.as_bytes()));
    }
    #[test]
    fn check_space() {
        let value = " ";
        assert_eq!("%20".as_bytes(), &encode(value.as_bytes()));
    }
    #[test]
    fn check_url_basic() {
        let value = "https://google.com/";
        assert_eq!(
            "https%3A%2F%2Fgoogle.com%2F".as_bytes(),
            &encode(value.as_bytes())
        );
    }
    #[test]
    fn check_url_params() {
        let value = "https://google.com?cookie=not available";
        assert_eq!(
            "https%3A%2F%2Fgoogle.com%3Fcookie%3Dnot%20available".as_bytes(),
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
                .iter()
                .map(|x| *x as char)
                .collect::<String>()
        );
    }
}
