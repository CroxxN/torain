#!/usr/bin/env python3
"""Send a BitTorrent DHT KRPC ping to router.bittorrent.com:6881."""

"""AI-GENERATED"""

import random
import socket

HOST = "dht.transmissionbt.com"
PORT = 6881
TIMEOUT_SECONDS = 60


def bencode(value):
    if isinstance(value, int):
        return f"i{value}e".encode()
    if isinstance(value, bytes):
        return str(len(value)).encode() + b":" + value
    if isinstance(value, str):
        return bencode(value.encode())
    if isinstance(value, dict):
        return (
            b"d"
            + b"".join(bencode(key) + bencode(value[key]) for key in sorted(value))
            + b"e"
        )
    if isinstance(value, list):
        return b"l" + b"".join(bencode(item) for item in value) + b"e"
    raise TypeError(f"cannot bencode {type(value).__name__}")


def bdecode(data, index=0):
    token = data[index : index + 1]
    if token == b"i":
        end = data.index(b"e", index)
        return int(data[index + 1 : end]), end + 1
    if token == b"l":
        items = []
        index += 1
        while data[index : index + 1] != b"e":
            item, index = bdecode(data, index)
            items.append(item)
        return items, index + 1
    if token == b"d":
        result = {}
        index += 1
        while data[index : index + 1] != b"e":
            key, index = bdecode(data, index)
            result[key], index = bdecode(data, index)
        return result, index + 1
    if token.isdigit():
        colon = data.index(b":", index)
        length = int(data[index:colon])
        start = colon + 1
        return data[start : start + length], start + length
    raise ValueError(f"invalid bencode at byte {index}")


def random_node_id():
    # A 20-byte ID is required by the DHT protocol. This is suitable for a ping.
    return random.randbytes(20)


def main():
    transaction_id = random.randbytes(2)
    request = {
        b"t": transaction_id,
        b"y": b"q",
        b"q": b"ping",
        b"a": {b"id": random_node_id()},
    }

    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
        sock.settimeout(TIMEOUT_SECONDS)
        sock.sendto(bencode(request), (HOST, PORT))
        print(f"Sent DHT ping to {HOST}:{PORT}")

        try:
            response, address = sock.recvfrom(4096)
        except TimeoutError:
            print(f"No response within {TIMEOUT_SECONDS} seconds")
            return 1

    decoded, _ = bdecode(response)
    if decoded.get(b"t") != transaction_id:
        print("Received a response with an unexpected transaction ID")
        return 1

    if decoded.get(b"y") == b"r":
        print(f"Received valid DHT response from {address[0]}:{address[1]}")
        print(f"Remote node ID: {decoded.get(b'r', {}).get(b'id', b'<missing>')!r}")
        return 0

    print(f"Received DHT error/unknown response: {decoded!r}")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
