use sha2::Digest;

use super::HashAlgorithm;

pub struct Sha256Hash;

impl HashAlgorithm for Sha256Hash {
    fn name(&self) -> &'static str {
        "sha256"
    }

    fn hex_length(&self) -> usize {
        64
    }

    fn hash(&self, input: &[u8]) -> String {
        hex::encode(sha2::Sha256::digest(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_hash() {
        let h = Sha256Hash;
        assert_eq!(
            h.hash(b"password"),
            "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8"
        );
    }
}
