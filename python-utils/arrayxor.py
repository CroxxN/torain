def xor(a, b):
    x = int.from_bytes(a, "big")
    y = int.from_bytes(b, "big")
    return x ^ y


def main():
    a = [
        250,
        114,
        245,
        235,
        33,
        112,
        17,
        131,
        196,
        39,
        97,
        86,
        89,
        81,
        16,
        84,
        116,
        88,
        58,
        86,
    ]
    b = [
        219,
        58,
        81,
        117,
        232,
        139,
        159,
        156,
        104,
        107,
        30,
        255,
        223,
        107,
        47,
        219,
        96,
        226,
        28,
        142,
    ]
    c = xor(a, b)
    print(c)


if __name__ == "__main__":
    main()
