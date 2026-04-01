#![allow(dead_code)]

use crypto::sha1::Sha1;
use std::time;

/// Token rotation interval: 10 minutes
const TOKEN_ROTATION_SECS: u64 = 10 * 60;

/// Generates and validates tokens for announce_peer requests.
/// Uses SHA-1(ip_bytes + secret) where the secret rotates periodically.
/// Keeps two secrets (current + previous) to allow for in-flight requests
/// that were issued with the old token.
#[derive(Debug)]
pub struct TokenManager {
    current_secret: [u8; 20],
    previous_secret: [u8; 20],
    last_rotation: time::Instant,
}

impl TokenManager {
    pub fn new() -> Self {
        let secret = Self::generate_secret();
        Self {
            current_secret: secret,
            previous_secret: secret,
            last_rotation: time::Instant::now(),
        }
    }

    /// Generate a token for the given IP address bytes.
    /// Call rotate() before generating to ensure freshness.
    pub fn generate(&mut self, ip: &[u8]) -> Vec<u8> {
        self.maybe_rotate();
        Self::hash_token(ip, &self.current_secret)
    }

    /// Validate that a token is valid for the given IP address.
    /// Accepts tokens generated with either the current or previous secret.
    pub fn validate(&mut self, token: &[u8], ip: &[u8]) -> bool {
        self.maybe_rotate();
        let current = Self::hash_token(ip, &self.current_secret);
        let previous = Self::hash_token(ip, &self.previous_secret);
        token == current.as_slice() || token == previous.as_slice()
    }

    /// Rotate secrets if enough time has passed
    fn maybe_rotate(&mut self) {
        let elapsed = time::Instant::now().duration_since(self.last_rotation);
        if elapsed >= time::Duration::from_secs(TOKEN_ROTATION_SECS) {
            self.previous_secret = self.current_secret;
            self.current_secret = Self::generate_secret();
            self.last_rotation = time::Instant::now();
        }
    }

    /// Force a rotation (useful for testing)
    pub fn force_rotate(&mut self) {
        self.previous_secret = self.current_secret;
        self.current_secret = Self::generate_secret();
        self.last_rotation = time::Instant::now();
    }

    fn generate_secret() -> [u8; 20] {
        let mut sha1 = Sha1::new();
        let seed = time::SystemTime::now()
            .duration_since(time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let seed_val = seed.as_nanos() as u64;
        let rand_val = crypto::tinymt::TinyMT::rand(seed_val as u32).get_u32();
        let combined = (rand_val as u64).wrapping_mul(seed_val);
        sha1.append_hash(&combined.to_be_bytes());
        sha1.get_hash()
    }

    fn hash_token(ip: &[u8], secret: &[u8; 20]) -> Vec<u8> {
        let mut sha1 = Sha1::new();
        sha1.append_hash(ip);
        sha1.append_hash(secret);
        sha1.get_hash().to_vec()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_and_validate() {
        let mut tm = TokenManager::new();
        let ip = [127, 0, 0, 1];
        let token = tm.generate(&ip);
        assert!(tm.validate(&token, &ip));
    }

    #[test]
    fn different_ip_fails_validation() {
        let mut tm = TokenManager::new();
        let ip1 = [127, 0, 0, 1];
        let ip2 = [192, 168, 1, 1];
        let token = tm.generate(&ip1);
        assert!(!tm.validate(&token, &ip2));
    }

    #[test]
    fn previous_secret_still_valid() {
        let mut tm = TokenManager::new();
        let ip = [10, 0, 0, 1];
        let token = tm.generate(&ip);

        // force rotation
        tm.force_rotate();

        // old token should still be valid (using previous_secret)
        assert!(tm.validate(&token, &ip));
    }

    #[test]
    fn two_rotations_invalidates() {
        let mut tm = TokenManager::new();
        let ip = [10, 0, 0, 1];
        let token = tm.generate(&ip);

        // two rotations: the original secret is now gone
        tm.force_rotate();
        tm.force_rotate();

        assert!(!tm.validate(&token, &ip));
    }
}
