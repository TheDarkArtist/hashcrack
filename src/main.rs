mod checkpoint;
mod cli;
mod cracker;
mod error;
mod hash;
mod output;
mod progress;
mod rules;
mod wordlist;

use std::process;

use clap::Parser;

use cli::Args;
use error::{AppError, ExitCode};

fn main() {
    let args = Args::parse();

    match run(args) {
        Ok(found) => {
            let code = if found {
                ExitCode::Found
            } else {
                ExitCode::NotFound
            };
            process::exit(code as i32);
        }
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(ExitCode::Error as i32);
        }
    }
}

/// Main application logic. Returns Ok(true) if at least one hash was cracked.
fn run(args: Args) -> error::Result<bool> {
    // Configure thread pool
    if let Some(threads) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .ok();
    }

    // Handle benchmark mode
    if args.benchmark {
        return run_benchmark();
    }

    // Resolve algorithm
    let registry = hash::Registry::new();
    let algorithm = resolve_algorithm(&args, &registry)?;

    // Load targets
    let targets = load_targets(&args)?;
    if targets.is_empty() {
        return Err(AppError::NoTargets);
    }

    // Parse rules
    let rule_chain = match &args.rules {
        Some(names) => rules::parse_rules(names)?,
        None => rules::RuleChain::empty(),
    };

    // Load wordlist (with resume offset if applicable)
    let words = match &args.resume {
        Some(path) => {
            let cp = checkpoint::load(path)?;
            eprintln!("Resuming from line {}", cp.line_offset);
            wordlist::load_from_offset(&args.wordlist, cp.line_offset)?
        }
        None => wordlist::load(&args.wordlist)?,
    };

    // Set up progress tracker
    let progress = if args.format == cli::OutputFormat::Quiet {
        progress::ProgressTracker::hidden()
    } else if args.wordlist == "-" {
        progress::ProgressTracker::spinner()
    } else {
        progress::ProgressTracker::with_total(words.len() as u64)
    };

    // Run cracker
    let config = cracker::CrackerConfig {
        algorithm,
        rules: &rule_chain,
        salt_position: args.salt_position,
        ignore_case: args.ignore_case,
    };

    let results = cracker::crack(&words, targets, &config, &progress);

    let elapsed = std::time::Duration::from_secs_f64(progress.elapsed_secs());
    progress.finish("done");

    // Format output
    let formatter = output::create_formatter(args.format);

    for found in &results.found {
        formatter.found(found);
    }
    for hash in &results.not_found {
        formatter.not_found(hash);
    }

    let target_count = results.found.len() + results.not_found.len();
    formatter.summary(&output::CrackSummary {
        total_candidates: results.candidates_tested,
        found_count: results.found.len(),
        target_count,
        duration: elapsed,
        speed: progress.speed(),
    });

    Ok(!results.found.is_empty())
}

fn resolve_algorithm<'a>(
    args: &Args,
    registry: &'a hash::Registry,
) -> error::Result<&'a dyn hash::HashAlgorithm> {
    if let Some(algo) = &args.algo {
        return Ok(registry.get(algo));
    }

    // Auto-detect from hash length
    let hash_str = if let Some(h) = &args.hash {
        h.as_str()
    } else if let Some(path) = &args.hashes_file {
        // Peek at first line to detect algorithm
        let content = std::fs::read_to_string(path).map_err(|e| AppError::HashesFileRead {
            path: path.clone(),
            source: e,
        })?;
        let first_line = content.lines().next().unwrap_or("");
        let hash_part = first_line.split(':').next().unwrap_or(first_line);
        return registry.detect_by_length(hash_part.trim().len());
    } else {
        return Err(AppError::NoTargets);
    };

    let clean = hash_str.split(':').next().unwrap_or(hash_str).trim();
    hash::validate_hex(clean)?;
    registry.detect_by_length(clean.len())
}

fn load_targets(args: &Args) -> error::Result<Vec<cracker::CrackTarget>> {
    if let Some(hash) = &args.hash {
        Ok(vec![cracker::CrackTarget::parse(hash)])
    } else if let Some(path) = &args.hashes_file {
        let content = std::fs::read_to_string(path).map_err(|e| AppError::HashesFileRead {
            path: path.clone(),
            source: e,
        })?;
        let targets: Vec<_> = content
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(cracker::CrackTarget::parse)
            .collect();
        Ok(targets)
    } else {
        Err(AppError::NoTargets)
    }
}

fn run_benchmark() -> error::Result<bool> {
    let registry = hash::Registry::new();
    let iterations = 1_000_000;

    println!("Benchmarking {} iterations per algorithm...\n", iterations);

    for algo in registry.all() {
        let duration = cracker::benchmark(algo, iterations);
        let speed = iterations as f64 / duration.as_secs_f64();
        println!(
            "{:<8} {:.2}s ({:.0} hashes/sec)",
            algo.name(),
            duration.as_secs_f64(),
            speed,
        );
    }

    Ok(true)
}
