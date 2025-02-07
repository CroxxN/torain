// params
const MAT1_PARAM: u32 = 0x8f7011ee_u32;
const MAT2_PARAM: u32 = 0xfc78ff1f_u32;
const TMAT_PARAM: u32 = 0x3793fdff_u32;

// loops
const MIN_LOOP: u32 = 8;
const PRE_LOOP: u32 = 8;

// internal state constants
const SH0: u32 = 1;
const SH1: u32 = 10;
const SH8: u32 = 8;
const MASK: u32 = 0x7fffffff;

#[derive(Default)]
pub struct TinyMT {
    status: [u32; 4],
    mat1: u32,
    mat2: u32,
    tmat: u32,
}

impl TinyMT {
    /// Generate a random number from a seed value.
    /// `seed` can be anything.
    /// ```
    /// use tinymt::TinyMT;
    ///
    /// fn main(){
    ///     let rand = TinyMT::rand(1337);
    ///     println!("{}", rand);
    /// }
    /// ```
    pub fn rand(seed: u32) -> u32 {
        let mut tinymt = Self::default();

        tinymt.mat1 = MAT1_PARAM;
        tinymt.mat2 = MAT2_PARAM;
        tinymt.tmat = TMAT_PARAM;

        tinymt.status[0] = seed;
        tinymt.status[1] = MAT1_PARAM;
        tinymt.status[2] = MAT2_PARAM;
        tinymt.status[3] = TMAT_PARAM;

        for i in 1..MIN_LOOP {
            tinymt.status[i as usize & 3] ^= (i as u128
                + 1812433253_u32 as u128
                    * (tinymt.status[(i as usize - 1) & 3]
                        ^ (tinymt.status[(i as usize - 1) & 3] >> 30))
                        as u128) as u32;
        }

        for _ in 0..PRE_LOOP {
            tinymt.next_state();
        }

        tinymt.next_state();
        tinymt.temper()
    }

    fn next_state(&mut self) {
        let mut y = self.status[3];
        let mut x = (self.status[0] & MASK) ^ self.status[1] ^ self.status[2];

        x ^= x << SH0;
        y ^= (y >> SH1) ^ x;

        self.status[0] = self.status[1];
        self.status[1] = self.status[2];
        self.status[2] = x ^ (y << SH1);
        self.status[3] = y;

        if y & 1 > 0 {
            self.status[1] ^= self.mat1;
            self.status[2] ^= self.mat2;
        }
    }

    fn temper(&mut self) -> u32 {
        let mut t0 = self.status[3];
        let t1 = self.status[0] + (self.status[2] >> SH8);

        t0 ^= t1;

        if t1 & 1 > 0 {
            t0 ^= self.tmat;
        }

        t0
    }
}

#[cfg(test)]
mod test {
    use crate::TinyMT;

    #[test]
    fn rand() {
        let rand = TinyMT::rand(1);
        assert_eq!(rand, 1255019984);
    }

    #[test]
    fn rand_prime() {
        // 7823 is prime
        let rand = TinyMT::rand(7823);
        assert_eq!(rand, 4180267476);
    }
}
