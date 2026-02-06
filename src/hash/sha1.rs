use sha1::Digest;

use super::HashAlgorithm;

pub struct Sha1Hash;

impl HashAlgorithm for Sha1Hash {
    fn name(&self) -> &'static str {
        "sha1"
    }

    fn hex_length(&self) -> usize {
        40
    }

    fn hash(&self, input: &[u8]) -> String {
        hex::encode(sha1::Sha1::digest(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_hash() {
        let h = Sha1Hash;
        assert_eq!(
            h.hash(b"password"),
            "5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8"
        );
    }
}
