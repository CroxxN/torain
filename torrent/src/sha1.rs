pub static K: [u32; 4] = [0x5A827999, 0x6ED9EBA1, 0x8F1BBCDC, 0xCA62C1D6];

#[derive(Debug)]
pub enum Error {
    FailWrite,
}

use std::fmt::Write;

macro_rules! shift_rotate {
    ($num:literal ,$expression:expr) => {
        ($expression).rotate_left($num)
    };
}

pub struct Sha1 {
    num_blocks: u64,
    message_length: u64,
    f_buf: [u32; 5],
    h_buf: [u32; 5],
    word: [u32; 80],
}

impl Sha1 {
    pub const fn new() -> Self {
        Self {
            num_blocks: 0,
            message_length: 0,
            f_buf: [0; 5],
            h_buf: [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0],
            word: [0; 80],
        }
    }
    pub const fn get_hash(&self) -> [u32; 5] {
        self.h_buf
    }
    pub const fn get_words(&self, n: usize) -> u32 {
        self.word[n]
    }
    pub fn append_hash(&mut self, input: &[u8]) {
        self.message_length = (8 * input.len()) as u64;
        if input.len() < 56 {
            self.process_hash_last(input, true);
            return;
        }
        self.num_blocks = (input.len() / 64) as u64;
        let last_index_value = input.len() % 64;
        let mut curr_idx = 0;
        for _ in 0..self.num_blocks {
            self.process_hash_cont(&input[curr_idx..(curr_idx + 64)], false);
            curr_idx = curr_idx + 64;
        }
        if last_index_value < 56 {
            self.process_hash_last(&input[curr_idx..], true);
        } else {
            self.process_hash_cont(&input[curr_idx..], true);
            self.process_hash_last(&[0], false);
        }
    }
    pub const fn reset_hash(self) -> Self {
        Sha1::new()
    }
    fn initialize_bits(input: &[u8], flag: bool) -> [u8; 64] {
        let temp: &mut [u8; 64] = &mut [0; 64];
        temp.iter_mut().zip(0..input.len()).for_each(|(t, i)| {
            *t = input[i];
        });
        if flag {
            temp[input.len()] = 0x80;
        }
        temp.to_owned()
    }
    // Maybe you need to clean the words for each round trip of the message digest
    fn process_hash(&mut self, input: &[u8], flag: bool) {
        let temp = Self::initialize_bits(input, flag);
        let mut limit = input.len() / 4;
        if flag {
            limit = limit + 1;
        }
        self.word
            .iter_mut()
            // This is where the logic falters
            // ~fixed at 22:59
            .zip(0..std::cmp::max(limit, 14))
            .enumerate()
            .for_each(|(tlen, (word, _))| {
                let len = tlen * 4;
                *word = (temp[len + 3] as u32)
                    | (temp[len + 2] as u32) << 8
                    | (temp[len + 1] as u32) << 16
                    | (temp[len] as u32) << 24;
            });
        self.compute_hash();
        // self.hash(&temp);
    }
    fn process_hash_last(&mut self, _input: &[u8], flag: bool) {
        self.word[14] = (self.message_length >> 32) as u32;
        self.word[15] = (self.message_length & 0xFFFFFFFF) as u32;
        self.process_hash(_input, flag);
    }
    fn process_hash_cont(&mut self, _input: &[u8], flag: bool) {
        self.process_hash(_input, flag);
    }
    // TODO: Implement SHA-1 for files

    // pub fn initiate_file(&mut self, message: PathBuf) {
    //     let mut file = std::fs::File::open(message).unwrap();
    //     let mut message = Vec::new();
    //     file.read_to_end(&mut message).unwrap();
    //     let len = message.len();
    // }

    const fn f(&self, i: &usize) -> u32 {
        if *i < 20 {
            (self.f_buf[1] & self.f_buf[2]) | (!self.f_buf[1] & self.f_buf[3])
        } else if *i >= 40 && *i <= 59 {
            (self.f_buf[1] & self.f_buf[2])
                | (self.f_buf[1] & self.f_buf[3])
                | (self.f_buf[2] & self.f_buf[3])
        } else {
            self.f_buf[1] ^ self.f_buf[2] ^ self.f_buf[3]
        }
    }
    fn compute_hash(&mut self) {
        // for t in 0..16 {
        //     self.word[t] = message[t];
        // }
        for t in 16..80 {
            self.word[t] = shift_rotate!(
                1,
                self.word[t - 3] ^ self.word[t - 8] ^ self.word[t - 14] ^ self.word[t - 16]
            );
        }
        for t in 0..5 {
            self.f_buf[t] = self.h_buf[t]; // Works
                                           // self.f_buf[t].clone_from(&self.h_buf[t]);
        }
        for t in 0..80_usize {
            let mut temp: u32 = 0;
            let idx = ((t) / 20) as usize;
            temp = temp
                .wrapping_add(self.f_buf[0].rotate_left(5))
                .wrapping_add(self.f(&t))
                .wrapping_add(self.f_buf[4])
                .wrapping_add(self.word[t])
                .wrapping_add(K[idx]);
            self.f_buf[4] = self.f_buf[3];
            self.f_buf[3] = self.f_buf[2];
            self.f_buf[2] = self.f_buf[1].rotate_left(30);
            self.f_buf[1] = self.f_buf[0];
            self.f_buf[0] = temp;
        }
        for t in 0..5 {
            self.h_buf[t] = self.h_buf[t].wrapping_add(self.f_buf[t]);
        }
    }
    pub fn get_ascii_hash(&self) -> Result<String, Error> {
        let mut ascii_hash = String::new();
        let res = self
            .get_hash()
            .iter()
            .try_for_each(|x| write!(&mut ascii_hash, "{:x}", *x));
        if res.is_err() {
            return Err(Error::FailWrite);
        }
        Ok(ascii_hash)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    // Passes
    #[test]
    fn normal() {
        let mut sha = Sha1::new();
        sha.append_hash(b"abc");
        assert_eq!(
            sha.get_hash(),
            [0xA9993E36, 0x4706816A, 0xBA3E2571, 0x7850C26C, 0x9CD0D89D]
        );
    }

    #[test]
    fn ascii_hash() {
        let mut sha = Sha1::new();
        sha.append_hash(b"abc");
        // sha.append_hash(b"c");
        assert_eq!(
            sha.get_ascii_hash().unwrap(),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
    }

    #[test]
    fn only_a() {
        let mut sha = Sha1::new();
        sha.append_hash(b"a");
        assert_eq!(
            sha.get_hash(),
            [0x86f7e437, 0xfaa5a7fc, 0xe15d1ddc, 0xb9eaeaea, 0x377667b8]
        );
    }

    #[test]
    fn a_and_b() {
        let mut sha = Sha1::new();
        sha.append_hash(b"ab");
        assert_eq!(
            sha.get_hash(),
            [0xda23614e, 0x02469a0d, 0x7c7bd1bd, 0xab5c9c47, 0x4b1904dc]
        );
    }
    #[test]
    fn empty() {
        let mut sha = Sha1::new();
        sha.append_hash(b"");
        assert_eq!(
            sha.get_hash(),
            [0xda39a3ee, 0x5e6b4b0d, 0x3255bfef, 0x95601890, 0xafd80709]
        );
    }
    // Passes
    #[test]
    fn maximum_one_block() {
        let mut sha = Sha1::new();
        sha.append_hash(b"abcdbcdecdefdefgefghfghighijhijkijkljkl");
        assert_eq!(
            sha.get_hash(),
            [0xb5ed281b, 0xcb6242b2, 0x889eb9a9, 0xc1727f3e, 0x9ab6dac4]
        );
    }
    // Passes
    #[test]
    fn long_hash() {
        let mut sha = Sha1::new();
        sha.append_hash(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");
        assert_eq!(
            sha.get_hash(),
            [0x84983E44, 0x1C3BD26E, 0xBAAE4AA1, 0xF95129E5, 0xE54670F1]
        );
    }
    // Passes
    #[test]
    fn medium_hash() {
        let mut sha = Sha1::new();
        sha.append_hash(b"abcdefghijklmnopqrstuvwxyzabcdefghij");
        assert_eq!(
            sha.get_hash(),
            [0x6af10122, 0x75c9b513, 0x7b29ac2a, 0x5ef9a64c, 0x98e42635]
        );
    }
    // Passes
    #[test]
    fn long_long_hash() {
        let mut sha = Sha1::new();
        sha.append_hash(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopqabcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");
        assert_eq!(
            sha.get_hash(),
            [0xafc53a4e, 0xa20856f9, 0x8e08dc6f, 0x3a5c9833, 0x137768ed]
        );
    }
    // Passes
}
