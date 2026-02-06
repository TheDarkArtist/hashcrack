use sha2::Digest;

use super::HashAlgorithm;

pub struct Sha512Hash;

impl HashAlgorithm for Sha512Hash {
    fn name(&self) -> &'static str {
        "sha512"
    }

    fn hex_length(&self) -> usize {
        128
    }

    fn hash(&self, input: &[u8]) -> String {
        hex::encode(sha2::Sha512::digest(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_hash() {
        let h = Sha512Hash;
        assert_eq!(
            h.hash(b"password"),
            "b109f3bbbc244eb82441917ed06d618b9008dd09b3befd1b5e07394c706a8bb9\
             80b1d7785e5976ec049b46df5f1326af5a2ea6d103fd07c95385ffab0cacbc86"
        );
    }
}
