use std::time::Duration;

use crate::cli::OutputFormat;

/// Result of a successful crack.
pub struct CrackFound {
    pub hash: String,
    pub password: String,
}

/// Summary statistics after cracking completes.
pub struct CrackSummary {
    pub total_candidates: u64,
    pub found_count: usize,
    pub target_count: usize,
    pub duration: Duration,
    pub speed: f64,
}

/// Output formatting follows the Strategy pattern.
/// Each formatter has a single responsibility: one output style.
pub trait OutputFormatter {
    fn found(&self, result: &CrackFound);
    fn not_found(&self, hash: &str);
    fn summary(&self, stats: &CrackSummary);
}

pub fn create_formatter(format: OutputFormat) -> Box<dyn OutputFormatter> {
    match format {
        OutputFormat::Normal => Box::new(NormalFormatter),
        OutputFormat::Quiet => Box::new(QuietFormatter),
        OutputFormat::Verbose => Box::new(VerboseFormatter),
        OutputFormat::Json => Box::new(JsonFormatter),
    }
}

// --- Implementations ---

struct NormalFormatter;

impl OutputFormatter for NormalFormatter {
    fn found(&self, result: &CrackFound) {
        println!("{}:{}", result.hash, result.password);
    }

    fn not_found(&self, hash: &str) {
        println!("{}: not found", hash);
    }

    fn summary(&self, stats: &CrackSummary) {
        println!(
            "\n{}/{} hashes cracked in {:.2}s ({:.0} candidates/sec)",
            stats.found_count,
            stats.target_count,
            stats.duration.as_secs_f64(),
            stats.speed,
        );
    }
}

struct QuietFormatter;

impl OutputFormatter for QuietFormatter {
    fn found(&self, result: &CrackFound) {
        println!("{}", result.password);
    }

    fn not_found(&self, _hash: &str) {}

    fn summary(&self, _stats: &CrackSummary) {}
}

struct VerboseFormatter;

impl OutputFormatter for VerboseFormatter {
    fn found(&self, result: &CrackFound) {
        println!("[FOUND] {} => {}", result.hash, result.password);
    }

    fn not_found(&self, hash: &str) {
        println!("[MISS]  {}", hash);
    }

    fn summary(&self, stats: &CrackSummary) {
        println!("\n--- Summary ---");
        println!("Targets:    {}", stats.target_count);
        println!("Found:      {}", stats.found_count);
        println!("Candidates: {}", stats.total_candidates);
        println!("Duration:   {:.2}s", stats.duration.as_secs_f64());
        println!("Speed:      {:.0} candidates/sec", stats.speed);
    }
}

struct JsonFormatter;

impl OutputFormatter for JsonFormatter {
    fn found(&self, result: &CrackFound) {
        let json = serde_json::json!({
            "type": "found",
            "hash": result.hash,
            "password": result.password,
        });
        println!("{}", json);
    }

    fn not_found(&self, hash: &str) {
        let json = serde_json::json!({
            "type": "not_found",
            "hash": hash,
        });
        println!("{}", json);
    }

    fn summary(&self, stats: &CrackSummary) {
        let json = serde_json::json!({
            "type": "summary",
            "found": stats.found_count,
            "targets": stats.target_count,
            "candidates": stats.total_candidates,
            "duration_secs": stats.duration.as_secs_f64(),
            "speed": stats.speed,
        });
        println!("{}", json);
    }
}
