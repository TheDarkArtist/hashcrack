use std::collections::HashSet;
use std::sync::Mutex;

use rayon::prelude::*;

use crate::cli::SaltPosition;
use crate::hash::HashAlgorithm;
use crate::output::CrackFound;
use crate::progress::ProgressTracker;
use crate::rules::RuleChain;

/// A single hash target to crack, with optional salt.
pub struct CrackTarget {
    /// Normalized (lowercase) hash for comparison.
    pub hash: String,
    /// Optional salt extracted from hash:salt format.
    pub salt: Option<Vec<u8>>,
    /// Original input string for display purposes.
    pub original: String,
}

impl CrackTarget {
    /// Parse a target string. Supports plain hash or hash:salt format.
    pub fn parse(input: &str) -> Self {
        let trimmed = input.trim();
        if let Some((hash_part, salt_part)) = trimmed.split_once(':') {
            Self {
                hash: hash_part.to_lowercase(),
                salt: Some(salt_part.as_bytes().to_vec()),
                original: trimmed.to_string(),
            }
        } else {
            Self {
                hash: trimmed.to_lowercase(),
                salt: None,
                original: trimmed.to_string(),
            }
        }
    }
}

/// Configuration for a cracking session.
pub struct CrackerConfig<'a> {
    pub algorithm: &'a dyn HashAlgorithm,
    pub rules: &'a RuleChain,
    pub salt_position: SaltPosition,
    pub ignore_case: bool,
}

/// Results collected during cracking.
pub struct CrackResults {
    pub found: Vec<CrackFound>,
    pub not_found: Vec<String>,
    pub candidates_tested: u64,
}

/// Run the cracker against a list of words and targets.
/// Uses rayon for parallel iteration over the wordlist.
///
/// Design: this is a pure function (no self) that takes all dependencies
/// as parameters — easy to test, easy to reason about.
pub fn crack(
    words: &[String],
    targets: Vec<CrackTarget>,
    config: &CrackerConfig,
    progress: &ProgressTracker,
) -> CrackResults {
    // Build a HashSet of remaining targets for O(1) lookup.
    let remaining: Mutex<HashSet<String>> = Mutex::new(
        targets.iter().map(|t| t.hash.clone()).collect(),
    );

    // Map from hash -> (original_display, salt)
    let target_map: std::collections::HashMap<String, (&str, Option<&[u8]>)> = targets
        .iter()
        .map(|t| {
            (
                t.hash.clone(),
                (t.original.as_str(), t.salt.as_deref()),
            )
        })
        .collect();

    let found: Mutex<Vec<CrackFound>> = Mutex::new(Vec::new());

    words.par_iter().for_each(|word| {
        // Early exit if all targets found
        if remaining.lock().unwrap().is_empty() {
            return;
        }

        let candidates = config.rules.candidates(word);
        let candidate_count = candidates.len() as u64;

        for candidate in &candidates {
            // For each target, check if candidate matches.
            // We need to handle salted hashes per-target since salt differs.
            let remaining_guard = remaining.lock().unwrap();
            if remaining_guard.is_empty() {
                break;
            }

            // Compute hash for each unique salt situation
            let hashes_to_check: Vec<(String, String)> = remaining_guard
                .iter()
                .map(|target_hash| {
                    let salt = target_map
                        .get(target_hash)
                        .and_then(|(_, s)| *s);

                    let computed = match salt {
                        Some(s) => config.algorithm.hash_salted(
                            candidate.as_bytes(),
                            s,
                            config.salt_position,
                        ),
                        None => config.algorithm.hash(candidate.as_bytes()),
                    };

                    let computed = if config.ignore_case {
                        computed.to_lowercase()
                    } else {
                        computed
                    };

                    (target_hash.clone(), computed)
                })
                .collect();

            drop(remaining_guard);

            for (target_hash, computed) in hashes_to_check {
                if computed == target_hash {
                    let mut remaining_guard = remaining.lock().unwrap();
                    if remaining_guard.remove(&target_hash) {
                        found.lock().unwrap().push(CrackFound {
                            hash: target_map
                                .get(&target_hash)
                                .map(|(orig, _)| orig.to_string())
                                .unwrap_or(target_hash),
                            password: candidate.clone(),
                        });
                    }
                }
            }
        }

        progress.inc(candidate_count);
    });

    let found = found.into_inner().unwrap();
    let remaining = remaining.into_inner().unwrap();

    let not_found: Vec<String> = remaining.into_iter().collect();

    CrackResults {
        found,
        not_found,
        candidates_tested: progress.total_candidates(),
    }
}

/// Run benchmark: hash a test string N times per algorithm.
pub fn benchmark(
    algorithm: &dyn HashAlgorithm,
    iterations: u64,
) -> std::time::Duration {
    let input = b"benchmark_test_string_12345";
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        std::hint::black_box(algorithm.hash(input));
    }
    start.elapsed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash;

    #[test]
    fn crack_target_parses_plain_hash() {
        let target = CrackTarget::parse("abc123def");
        assert_eq!(target.hash, "abc123def");
        assert!(target.salt.is_none());
    }

    #[test]
    fn crack_target_parses_salted_hash() {
        let target = CrackTarget::parse("abc123:mysalt");
        assert_eq!(target.hash, "abc123");
        assert_eq!(target.salt.as_deref(), Some(b"mysalt".as_slice()));
    }

    #[test]
    fn crack_finds_password() {
        let registry = hash::Registry::new();
        let algo = registry.get(&crate::cli::Algorithm::Sha1);
        let rules = RuleChain::empty();
        let progress = ProgressTracker::hidden();

        let words = vec!["hello".to_string(), "password".to_string()];
        let targets = vec![CrackTarget::parse(
            "5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8",
        )];

        let config = CrackerConfig {
            algorithm: algo,
            rules: &rules,
            salt_position: SaltPosition::Prefix,
            ignore_case: false,
        };

        let results = crack(&words, targets, &config, &progress);
        assert_eq!(results.found.len(), 1);
        assert_eq!(results.found[0].password, "password");
    }

    #[test]
    fn crack_reports_not_found() {
        let registry = hash::Registry::new();
        let algo = registry.get(&crate::cli::Algorithm::Sha1);
        let rules = RuleChain::empty();
        let progress = ProgressTracker::hidden();

        let words = vec!["nope".to_string()];
        let targets = vec![CrackTarget::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")];

        let config = CrackerConfig {
            algorithm: algo,
            rules: &rules,
            salt_position: SaltPosition::Prefix,
            ignore_case: false,
        };

        let results = crack(&words, targets, &config, &progress);
        assert!(results.found.is_empty());
        assert_eq!(results.not_found.len(), 1);
    }
}
