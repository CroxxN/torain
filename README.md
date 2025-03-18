# A bit-torrent client. From the ground up.

It aims to be a complete torrent client with custom implementation of all useful libraries and tools. So far, torain contains a bencode serializer/deserializer, a sha1 hash library, a URL library, a HTTP/UDP request library, a Psuedo-Random Number Generator.

### Milestone

- [x] bencode parser
- [x] parse .torrent file
- [x] bencode encoder
- [x] sha1 info hash
- [x] URL Library
- [x] HTTP Library
- [x] TinyMT PRNG
- [ ] Bittorrent two-way web-server
- [ ] uTP Protocol
- [ ] DHT Library
- [ ] handle magnet links
- [ ] download files
- [ ] use custom Async executor instead of `tokio`

### References

- [The BitTorrent Protocol Specification](https://www.bittorrent.org/beps/bep_0003.html)
- [List of Bittorrent Extension Protocol](https://www.bittorrent.org/beps/bep_0000.html)
- [RFC 7574](https://www.rfc-editor.org/rfc/rfc7574.txt)
- [Unofficial Reference](https://wiki.theory.org/BitTorrentSpecification)
- [Kademlia Protocol](https://pdos.csail.mit.edu/~petar/papers/maymounkov-kademlia-lncs.pdf)
