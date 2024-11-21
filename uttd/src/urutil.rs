use std::{collections::HashMap, str::FromStr};

// #[allow(unused_variables, dead_code)]
// use crate::tracker::TrackerParams;

fn encode(value: &[u8]) -> String {
    let mut formatted = String::new();

    value.iter().for_each(|x| {
        formatted.push_str(&format!("%{:x}", x));
    });

    formatted
}

fn u8_string(value: &[u8]) -> String {
    value.iter().map(|x| *x as char).collect::<String>()
}

pub fn build_url(base: &str, params: &HashMap<&str, &[u8]>) -> String {
    let mut url: String = String::from_str(base).expect("FAILED to create String");

    url.push('?');

    for (k, v) in params {
        if *k == "info_hash" {
            url.push_str(k);
            url.push('=');
            url.push_str(&encode(v));
            url.push('&');
            continue;
        }
        url.push_str(k);
        url.push('=');
        url.push_str(&u8_string(v));
        url.push('&');
    }
    url.pop();

    url
}
