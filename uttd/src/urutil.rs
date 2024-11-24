use std::{collections::HashMap, str::FromStr};

use crate::{url::Scheme, UttdError};

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

pub fn build_url(base: &str, params: &HashMap<&str, Vec<u8>>) -> String {
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

pub fn response_body(scheme: Scheme, res: &mut Vec<u8>) -> Result<&mut [u8], UttdError> {
    match scheme {
        Scheme::UDP => response_body_udp(res),
        _ => response_body_tcp(res),
    }
}

pub fn response_body_udp(res: &mut Vec<u8>) -> Result<&mut [u8], UttdError> {
    let (_, response) = res.split_at_mut(20);

    let response_body = &mut response[20..];

    Ok(response_body)
    // Err(UttdError::FailedRequest)
}

pub fn response_body_tcp(res: &mut Vec<u8>) -> Result<&mut [u8], UttdError> {
    let response_code =
        (res[9] as u32 - 48) * 100 + (res[10] as u32 - 48) * 10 + (res[11] as u32 - 48);

    if response_code != 200 {
        return Err(UttdError::FailedRequest);
    };
    let mut count = 0;

    // find the "\r\n\r\n" and split it off, second &[u8] contains the response body
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Messages

    for (pos, bytes) in res.iter().enumerate() {
        if count == 4 {
            count = pos;
            break;
        }
        if *bytes == b'\r' || *bytes == b'\n' {
            count += 1;
        } else {
            count = 0;
        }
    }
    let (_, body) = res.split_at_mut(count);

    Ok(body)
}

#[cfg(test)]

mod test {

    use super::response_body;

    #[test]
    fn parse_ok() {
        let mut value = "HTTP/1.1 200 OK\r\nServer: mimosa\r\nConnection: Close\r\nContent-Length: 338\r\nContent-Type: text/plain\r\n\r\nd8:intervali900e5:peers300:°e\u{84}¾Èy¹A\u{86}¾\u{1a}ák\u{9f}ë\u{11}ÈÕ&Fù*¦\u{89}hõOÕ\u{1a}áPC³;ÈÕ\u{92}F¦ÚW¡³¸7cÇ\u{1a}m½¶íÃ`Y\u{95}Åå®ÍÁ \u{7f}ì8vY:\u{f}TåÞ\u{8f}ô/TØ\u{80}\u{2}Sq°ÈÕO\u{7f}Ïªð5YéÏoÀO¼ó%§\u{1a}álÁ\u{9e}\u{96}`ì_\u{18}0ìÙ<\u{5}&É\u{8d}ÈÕÆ6\u{80}|)FµªV³áUZÿôÛÈ\"1\u{90}&>+ÊÉ2{\"ÈÕ-\u{8c}¸\u{1a}\u{95}\u{15}©\u{96}Åb\u{81}V%ä»1AñWÔÄ\u{1c}IrÙ\u{1f}¹åÈÕ¼º{\u{8d}\u{12}ù¸<4X'\u{11}V¡\u{9d}ÅÈÕR@³³ÈÖ\u{92}F³\u{1e}Rvµ×°Cså¨ã(ñ\u{1a}á°;`\0áMÁË\rÇ\u{91}Ñ±¾\u{9e}Ò\u{1a}á\u{90}\u{2}A_NÐ\u{18}\u{4}\u{7}1\u{1a}ák\u{89}Ã×\u{1a}áG¡n\\ê`R¥æ½ØØ§rÎÉÊ5\u{1b}*a¨\u{1a}âU\u{86}\u{8}\u{c}ÈÕ%ûh\u{a0}\u{93}+\u{90}¬±ªÈÖ6:peers60:e".as_bytes().to_vec();

        let body = "d8:intervali900e5:peers300:°e\u{84}¾Èy¹A\u{86}¾\u{1a}ák\u{9f}ë\u{11}ÈÕ&Fù*¦\u{89}hõOÕ\u{1a}áPC³;ÈÕ\u{92}F¦ÚW¡³¸7cÇ\u{1a}m½¶íÃ`Y\u{95}Åå®ÍÁ \u{7f}ì8vY:\u{f}TåÞ\u{8f}ô/TØ\u{80}\u{2}Sq°ÈÕO\u{7f}Ïªð5YéÏoÀO¼ó%§\u{1a}álÁ\u{9e}\u{96}`ì_\u{18}0ìÙ<\u{5}&É\u{8d}ÈÕÆ6\u{80}|)FµªV³áUZÿôÛÈ\"1\u{90}&>+ÊÉ2{\"ÈÕ-\u{8c}¸\u{1a}\u{95}\u{15}©\u{96}Åb\u{81}V%ä»1AñWÔÄ\u{1c}IrÙ\u{1f}¹åÈÕ¼º{\u{8d}\u{12}ù¸<4X'\u{11}V¡\u{9d}ÅÈÕR@³³ÈÖ\u{92}F³\u{1e}Rvµ×°Cså¨ã(ñ\u{1a}á°;`\0áMÁË\rÇ\u{91}Ñ±¾\u{9e}Ò\u{1a}á\u{90}\u{2}A_NÐ\u{18}\u{4}\u{7}1\u{1a}ák\u{89}Ã×\u{1a}áG¡n\\ê`R¥æ½ØØ§rÎÉÊ5\u{1b}*a¨\u{1a}âU\u{86}\u{8}\u{c}ÈÕ%ûh\u{a0}\u{93}+\u{90}¬±ªÈÖ6:peers60:e".as_bytes();

        let res = response_body(crate::url::Scheme::HTTP, &mut value);

        assert!(res.is_ok());
        assert_eq!(res.unwrap(), body);
    }

    #[test]
    fn parse_err() {
        let mut value = "HTTP/1.1 301 Not Found".as_bytes().to_vec();
        let res = response_body(crate::url::Scheme::HTTP, &mut value);
        assert!(res.is_err());
    }
}
