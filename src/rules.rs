use crate::error::{AppError, Result};

/// A mutation rule generates candidate passwords from a base word.
/// Each rule has a single responsibility: one type of mutation.
pub trait MutationRule: Send + Sync {
    fn mutate(&self, word: &str) -> Vec<String>;
}

// --- Built-in rules ---

/// Appends digits 0-9 to the word.
struct AppendDigits;

impl MutationRule for AppendDigits {
    fn mutate(&self, word: &str) -> Vec<String> {
        (0..=9).map(|d| format!("{word}{d}")).collect()
    }
}

/// Produces case variations: original, UPPERCASE, Capitalized.
struct ToggleCase;

impl MutationRule for ToggleCase {
    fn mutate(&self, word: &str) -> Vec<String> {
        let upper = word.to_uppercase();
        let capitalized = capitalize(word);
        let mut results = Vec::with_capacity(3);
        // Only include variations that differ from the original
        if upper != word {
            results.push(upper);
        }
        if capitalized != word {
            results.push(capitalized);
        }
        results
    }
}

/// Common leet speak substitutions: a→4, e→3, i→1, o→0, s→5, t→7.
struct Leet;

impl MutationRule for Leet {
    fn mutate(&self, word: &str) -> Vec<String> {
        let leet: String = word
            .chars()
            .map(|c| match c {
                'a' | 'A' => '4',
                'e' | 'E' => '3',
                'i' | 'I' => '1',
                'o' | 'O' => '0',
                's' | 'S' => '5',
                't' | 'T' => '7',
                other => other,
            })
            .collect();
        if leet != word {
            vec![leet]
        } else {
            vec![]
        }
    }
}

/// Appends common suffixes: !, 123, 1234, @, #, 2024, 2025.
struct CommonSuffixes;

impl MutationRule for CommonSuffixes {
    fn mutate(&self, word: &str) -> Vec<String> {
        ["!", "123", "1234", "@", "#", "2024", "2025", "2026"]
            .iter()
            .map(|s| format!("{word}{s}"))
            .collect()
    }
}

/// Capitalizes the first letter.
struct Capitalize;

impl MutationRule for Capitalize {
    fn mutate(&self, word: &str) -> Vec<String> {
        let cap = capitalize(word);
        if cap != word { vec![cap] } else { vec![] }
    }
}

/// Reverses the word.
struct Reverse;

impl MutationRule for Reverse {
    fn mutate(&self, word: &str) -> Vec<String> {
        let rev: String = word.chars().rev().collect();
        if rev != word { vec![rev] } else { vec![] }
    }
}

// --- Rule chain ---

/// Composes multiple rules and yields all candidates for a word.
/// The original word is always included as the first candidate.
pub struct RuleChain {
    rules: Vec<Box<dyn MutationRule>>,
}

impl RuleChain {
    pub fn new(rules: Vec<Box<dyn MutationRule>>) -> Self {
        Self { rules }
    }

    pub fn empty() -> Self {
        Self { rules: vec![] }
    }

    /// Generate all candidates: original word + mutations from each rule.
    pub fn candidates(&self, word: &str) -> Vec<String> {
        let mut results = vec![word.to_string()];
        for rule in &self.rules {
            results.extend(rule.mutate(word));
        }
        results
    }

}

/// Parse comma-separated rule names from CLI into a RuleChain.
pub fn parse_rules(names: &[String]) -> Result<RuleChain> {
    let mut rules: Vec<Box<dyn MutationRule>> = Vec::new();
    for name in names {
        let rule: Box<dyn MutationRule> = match name.as_str() {
            "append_digits" => Box::new(AppendDigits),
            "toggle_case" => Box::new(ToggleCase),
            "leet" => Box::new(Leet),
            "common_suffixes" => Box::new(CommonSuffixes),
            "capitalize" => Box::new(Capitalize),
            "reverse" => Box::new(Reverse),
            other => return Err(AppError::UnknownRule(other.to_string())),
        };
        rules.push(rule);
    }
    Ok(RuleChain::new(rules))
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let upper: String = first.to_uppercase().collect();
            upper + chars.as_str()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_digits_produces_10() {
        let rule = AppendDigits;
        let results = rule.mutate("pass");
        assert_eq!(results.len(), 10);
        assert_eq!(results[0], "pass0");
        assert_eq!(results[9], "pass9");
    }

    #[test]
    fn toggle_case_variations() {
        let rule = ToggleCase;
        let results = rule.mutate("hello");
        assert!(results.contains(&"HELLO".to_string()));
        assert!(results.contains(&"Hello".to_string()));
    }

    #[test]
    fn leet_substitution() {
        let rule = Leet;
        let results = rule.mutate("password");
        assert_eq!(results, vec!["p455w0rd"]);
    }

    #[test]
    fn leet_noop_for_numbers() {
        let rule = Leet;
        let results = rule.mutate("12345");
        assert!(results.is_empty());
    }

    #[test]
    fn reverse_word() {
        let rule = Reverse;
        let results = rule.mutate("abc");
        assert_eq!(results, vec!["cba"]);
    }

    #[test]
    fn reverse_palindrome_is_empty() {
        let rule = Reverse;
        assert!(rule.mutate("aba").is_empty());
    }

    #[test]
    fn rule_chain_includes_original() {
        let chain = RuleChain::empty();
        let candidates = chain.candidates("word");
        assert_eq!(candidates, vec!["word"]);
    }

    #[test]
    fn parse_rules_rejects_unknown() {
        let result = parse_rules(&["bogus".to_string()]);
        assert!(result.is_err());
    }
}
