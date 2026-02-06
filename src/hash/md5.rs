use md5::Digest;

use super::HashAlgorithm;

pub struct Md5Hash;

impl HashAlgorithm for Md5Hash {
    fn name(&self) -> &'static str {
        "md5"
    }

    fn hex_length(&self) -> usize {
        32
    }

    fn hash(&self, input: &[u8]) -> String {
        hex::encode(md5::Md5::digest(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_hash() {
        let h = Md5Hash;
        assert_eq!(
            h.hash(b"password"),
            "5f4dcc3b5aa765d61d8327deb882cf99"
        );
    }
}
