use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(
    name = "hashcrack",
    version,
    about = "A multi-algorithm, multi-threaded hash cracking tool"
)]
pub struct Args {
    /// Path to wordlist file (use '-' for stdin)
    #[arg(value_name = "WORDLIST")]
    pub wordlist: String,

    /// Target hash to crack
    #[arg(value_name = "HASH", required_unless_present_any = ["hashes_file", "benchmark"])]
    pub hash: Option<String>,

    /// File containing hashes to crack (one per line, or hash:salt format)
    #[arg(long, value_name = "FILE", conflicts_with = "hash")]
    pub hashes_file: Option<PathBuf>,

    /// Hash algorithm (auto-detected from hash length if omitted)
    #[arg(short, long, value_enum)]
    pub algo: Option<Algorithm>,

    /// Number of threads (defaults to number of CPU cores)
    #[arg(short = 'j', long, value_name = "NUM")]
    pub threads: Option<usize>,

    /// Mutation rules to apply (comma-separated)
    /// Available: append_digits, toggle_case, leet, common_suffixes, capitalize, reverse
    #[arg(short, long, value_name = "RULES", value_delimiter = ',')]
    pub rules: Option<Vec<String>>,

    /// Resume from a checkpoint file
    #[arg(long, value_name = "FILE")]
    pub resume: Option<PathBuf>,

    /// Output format
    #[arg(long, value_enum, default_value = "normal")]
    pub format: OutputFormat,

    /// Case-insensitive hash comparison
    #[arg(long)]
    pub ignore_case: bool,

    /// Run benchmark mode (test hashing speed per algorithm)
    #[arg(long)]
    pub benchmark: bool,

    /// Salt position for salted hashes
    #[arg(long, value_enum, default_value = "prefix")]
    pub salt_position: SaltPosition,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum Algorithm {
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Normal,
    Quiet,
    Verbose,
    Json,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaltPosition {
    Prefix,
    Suffix,
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Algorithm::Md5 => write!(f, "md5"),
            Algorithm::Sha1 => write!(f, "sha1"),
            Algorithm::Sha256 => write!(f, "sha256"),
            Algorithm::Sha512 => write!(f, "sha512"),
        }
    }
}
