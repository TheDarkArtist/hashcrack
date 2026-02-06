use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

use crate::error::{AppError, Result};

/// Reads a wordlist from either a file path or stdin.
/// Returns all lines loaded into memory for parallel processing.
pub fn load(path: &str) -> Result<Vec<String>> {
    let reader: Box<dyn Read> = if path == "-" {
        Box::new(io::stdin().lock())
    } else {
        let file = File::open(path).map_err(|e| AppError::WordlistRead {
            path: Path::new(path).to_path_buf(),
            source: e,
        })?;
        Box::new(file)
    };

    let buf = BufReader::new(reader);
    let mut words = Vec::new();

    for line in buf.lines() {
        let line = line.map_err(|e| AppError::WordlistRead {
            path: Path::new(path).to_path_buf(),
            source: e,
        })?;
        let trimmed = line.trim().to_string();
        if !trimmed.is_empty() {
            words.push(trimmed);
        }
    }

    Ok(words)
}

/// Load words starting from a specific line offset (for resume support).
pub fn load_from_offset(path: &str, offset: usize) -> Result<Vec<String>> {
    let mut words = load(path)?;
    if offset < words.len() {
        words.drain(..offset);
    } else {
        words.clear();
    }
    Ok(words)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn load_skips_empty_lines() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_wordlist.txt");
        {
            let mut f = File::create(&path).unwrap();
            writeln!(f, "alpha").unwrap();
            writeln!(f).unwrap();
            writeln!(f, "  beta  ").unwrap();
            writeln!(f, "gamma").unwrap();
        }
        let words = load(path.to_str().unwrap()).unwrap();
        assert_eq!(words, vec!["alpha", "beta", "gamma"]);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn load_from_offset_skips_lines() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_wordlist_offset.txt");
        {
            let mut f = File::create(&path).unwrap();
            writeln!(f, "a").unwrap();
            writeln!(f, "b").unwrap();
            writeln!(f, "c").unwrap();
        }
        let words = load_from_offset(path.to_str().unwrap(), 2).unwrap();
        assert_eq!(words, vec!["c"]);
        std::fs::remove_file(&path).ok();
    }
}
