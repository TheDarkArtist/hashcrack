use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use indicatif::{ProgressBar, ProgressStyle};

/// Thread-safe progress tracker wrapping indicatif.
/// Separated from cracking logic to keep progress display concerns isolated.
pub struct ProgressTracker {
    bar: ProgressBar,
    start: Instant,
    candidates: AtomicU64,
}

impl ProgressTracker {
    /// Create a progress bar with a known total (file-based wordlist).
    pub fn with_total(total: u64) -> Self {
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] \
                 {pos}/{len} ({per_sec}) ETA: {eta}",
            )
            .unwrap()
            .progress_chars("=>-"),
        );
        Self {
            bar,
            start: Instant::now(),
            candidates: AtomicU64::new(0),
        }
    }

    /// Create a spinner for unknown total (stdin).
    pub fn spinner() -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {pos} candidates ({per_sec})")
                .unwrap(),
        );
        Self {
            bar,
            start: Instant::now(),
            candidates: AtomicU64::new(0),
        }
    }

    /// Create a no-op tracker for quiet mode.
    pub fn hidden() -> Self {
        let bar = ProgressBar::hidden();
        Self {
            bar,
            start: Instant::now(),
            candidates: AtomicU64::new(0),
        }
    }

    pub fn inc(&self, delta: u64) {
        self.candidates.fetch_add(delta, Ordering::Relaxed);
        self.bar.inc(delta);
    }

    pub fn finish(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }

    pub fn total_candidates(&self) -> u64 {
        self.candidates.load(Ordering::Relaxed)
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    pub fn speed(&self) -> f64 {
        let elapsed = self.elapsed_secs();
        if elapsed > 0.0 {
            self.total_candidates() as f64 / elapsed
        } else {
            0.0
        }
    }

}
