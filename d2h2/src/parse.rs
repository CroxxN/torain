use bencode::bencode::BTypes;
use uttd::url::Url;

#[derive(Debug)]
struct Response<'a> {
    id: &'a [u8],
    node: Vec<Url>,
}

impl<'a> Response<'a> {
    fn new(packet: &'a [u8]) -> Self {
        let decoded = bencode::bencode::decode(&mut packet.into_iter().map(|x| *x))
            .expect("Unable to decode bytes");

        if let BTypes::DICT(d) = decoded {
            if let Some(y) = d.get("y") {
                let status: String = y.try_into().unwrap();
                if &status == "e" {
                    // can use unwrap here because we know for certain that the request has errored
                    let err_code: usize = d.get("e").unwrap().try_into().unwrap();
                    match err_code {
                        201 => (),
                        202 => (),
                        203 => (),
                        204 => (),
                        _ => (),
                    };
                    todo!()
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
                    }
                }
                // return ();
                todo!()
            } else {
                panic!("Key 'y' not present.");
            }
        }

        todo!()
    }
}
