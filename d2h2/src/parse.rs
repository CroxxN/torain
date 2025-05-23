use bencode::bencode::BTypes;
use uttd::url::Url;

use crate::error::DHTError;
use crate::utils::*;

#[derive(Debug)]
struct Response<'a> {
    id: &'a [u8],
    node: Vec<Url>,
}

impl<'a> Response<'a> {
    fn new(packet: &'a [u8]) -> Result<Self, DHTError> {
        // decoded packet
        let decoded =
            bencode::bencode::decode(&mut packet.iter().copied()).expect("Unable to decode bytes");

        // key: y
        // TODO: move the y-key checking logic to `get_string_from_dict`
        // "y" key can take two values: "e" and "q". "y: 'e'" suggests that an error has occured.
        // The error can be found in key "e".
        // "y: 'q'" suggests success. Look at key "q" for further steps.

        if let Some(v) = get_string_from_dict(&decoded, "y") {
            if &v == "e" {
                let err_code: usize = get_usize_from_dict(&decoded, "e").unwrap();
                return Err(DHTError::Error(err_code as u32));
            }
        }

        if let BTypes::DICT(d) = decoded {
            if let Some(y) = d.get("y") {
                let status: String = y.try_into().unwrap();
                if &status == "e" {
                    // can use unwrap here because we know for certain that the request has errored
                    let err_code: usize = d.get("e").unwrap().try_into().unwrap();
                    return Err(DHTError::Error(err_code as u32));
                }

                if let Some(response) = d.get("r") {
                    if let BTypes::DICT(d) = response {
                        let nodes: &[u8] = d.get("nodes").unwrap().try_into().unwrap();
                        let mut urls = Vec::with_capacity(8);
                        for x in nodes.chunks_exact(6) {
                            let port = u16::from_be_bytes([x[4], x[5]]);
                            let url = Url::from_ip_bytes(&x[..4], port);
                            urls.push(url);
                        }
                        return Ok(Self {
                            id: &[128],
                            node: urls,
                        });
                    }
                }
                // return ();
                todo!()
            } else {
                panic!("Key 'y' not present.");
            }
        } else {
            Err(DHTError::Error(10))
        }
    }
}
