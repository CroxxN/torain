// #[allow(unused_variables, dead_code)]
use crate::tracker::TrackerParams;

pub fn encode(value: &[u8]) -> String {
    let mut formatted = String::new();

    value.iter().for_each(|x| match *x {
        b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'.' | b'-' | b'_' => formatted.push(*x as char),

        _ => formatted.push_str(&format!("%{:X}", x)),
    });

    formatted
}

pub fn build_init_url(base: &str, tracker_param: TrackerParams) {
    let mut url_space: Vec<u8> = base.as_bytes().iter().map(|x| *x).collect();

    // info_hash
    url_space.push(b'?');
    url_space.extend("info_hash".as_bytes());
    url_space.push(b'=');
    url_space.extend(tracker_param.info_hash);

    url_space.push(b'&');

    // peer_id
    url_space.extend("peer_id".as_bytes());
    url_space.push(b'=');
    url_space.extend(tracker_param.peer_id);

    url_space.push(b'&');

    // port
    url_space.extend("port".as_bytes());
    url_space.push(b'=');
    url_space.extend(tracker_param.port);

    url_space.push(b'&');

    // uploaded
    url_space.extend("uploaded".as_bytes());
    url_space.push(b'=');
    url_space.extend(tracker_param.uploaded);

    url_space.push(b'&');

    // downloaded
    url_space.extend("downloaded".as_bytes());
    url_space.push(b'=');
    url_space.extend(tracker_param.downloaded);

    url_space.push(b'&');

    // left
    url_space.extend("left".as_bytes());
    url_space.push(b'=');
    url_space.extend(tracker_param.left);

    url_space.push(b'&');

    // compact
    url_space.extend("compact".as_bytes());
    url_space.push(b'=');
    url_space.push(tracker_param.compact);

    url_space.push(b'&');

    // no_peer_id
    url_space.extend("no_peer_id".as_bytes());
    url_space.push(b'=');
    url_space.push(1);

    url_space.push(b'&');

    // event
    url_space.extend("event".as_bytes());
    url_space.push(b'=');
    url_space.extend("started".as_bytes());

    url_space.push(b'&');
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
