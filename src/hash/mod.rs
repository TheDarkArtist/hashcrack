mod md5;
mod sha1;
mod sha256;
mod sha512;

use crate::cli::{Algorithm, SaltPosition};
use crate::error::{AppError, Result};

/// Core abstraction: any hash algorithm must implement this trait.
/// Open for extension (new algorithms) without modifying existing code.
pub trait HashAlgorithm: Send + Sync {
    fn name(&self) -> &'static str;
    fn hex_length(&self) -> usize;
    fn hash(&self, input: &[u8]) -> String;

    fn hash_salted(&self, input: &[u8], salt: &[u8], position: SaltPosition) -> String {
        match position {
            SaltPosition::Prefix => {
                let mut combined = salt.to_vec();
                combined.extend_from_slice(input);
                self.hash(&combined)
            }
            SaltPosition::Suffix => {
                let mut combined = input.to_vec();
                combined.extend_from_slice(salt);
                self.hash(&combined)
            }
        }
    }
}

/// Registry of all supported hash algorithms.
/// Single source of truth for algorithm lookup and auto-detection.
pub struct Registry {
    algorithms: Vec<Box<dyn HashAlgorithm>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            algorithms: vec![
                Box::new(md5::Md5Hash),
                Box::new(sha1::Sha1Hash),
                Box::new(sha256::Sha256Hash),
                Box::new(sha512::Sha512Hash),
            ],
        }
    }

    /// Look up algorithm by CLI enum variant.
    pub fn get(&self, algo: &Algorithm) -> &dyn HashAlgorithm {
        let name = algo.to_string();
        self.algorithms
            .iter()
            .find(|a| a.name() == name)
            .map(|a| a.as_ref())
            .expect("all Algorithm variants must be registered")
    }

    /// Auto-detect algorithm from hex hash length.
    /// Returns error if length doesn't match any known algorithm.
    pub fn detect_by_length(&self, hex_len: usize) -> Result<&dyn HashAlgorithm> {
        let matches: Vec<_> = self
            .algorithms
            .iter()
            .filter(|a| a.hex_length() == hex_len)
            .collect();

        match matches.len() {
            0 => Err(AppError::AmbiguousHash(hex_len)),
            1 => Ok(matches[0].as_ref()),
            _ => Err(AppError::AmbiguousHash(hex_len)),
        }
    }

    /// List all registered algorithms (for benchmark mode).
    pub fn all(&self) -> impl Iterator<Item = &dyn HashAlgorithm> {
        self.algorithms.iter().map(|a| a.as_ref())
    }
}

/// Validate that a string is valid hexadecimal.
pub fn validate_hex(s: &str) -> Result<()> {
    if s.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(AppError::InvalidHash {
            reason: format!("contains non-hex characters: {s}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_detects_md5() {
        let reg = Registry::new();
        let algo = reg.detect_by_length(32).unwrap();
        assert_eq!(algo.name(), "md5");
    }

    #[test]
    fn registry_detects_sha1() {
        let reg = Registry::new();
        let algo = reg.detect_by_length(40).unwrap();
        assert_eq!(algo.name(), "sha1");
    }

    #[test]
    fn registry_detects_sha256() {
        let reg = Registry::new();
        let algo = reg.detect_by_length(64).unwrap();
        assert_eq!(algo.name(), "sha256");
    }

    #[test]
    fn registry_detects_sha512() {
        let reg = Registry::new();
        let algo = reg.detect_by_length(128).unwrap();
        assert_eq!(algo.name(), "sha512");
    }

    #[test]
    fn registry_rejects_unknown_length() {
        let reg = Registry::new();
        assert!(reg.detect_by_length(99).is_err());
    }

    #[test]
    fn validate_hex_accepts_valid() {
        assert!(validate_hex("5baa61e4c9b93f3f").is_ok());
        assert!(validate_hex("ABCDEF0123456789").is_ok());
    }

    #[test]
    fn validate_hex_rejects_invalid() {
        assert!(validate_hex("xyz123").is_err());
        assert!(validate_hex("5baa61g4").is_err());
    }
}
