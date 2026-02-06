use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("invalid hash: {reason}")]
    InvalidHash { reason: String },

    #[error("could not detect algorithm for hash length {0} — use --algo to specify")]
    AmbiguousHash(usize),

    #[error("no targets provided — pass a hash or use --hashes-file")]
    NoTargets,

    #[error("failed to read wordlist: {path}")]
    WordlistRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read hashes file: {path}")]
    HashesFileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("checkpoint error: {reason}")]
    Checkpoint { reason: String },

    #[error("unknown rule: {0}")]
    UnknownRule(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;

/// Process exit codes following Unix conventions.
pub enum ExitCode {
    Found = 0,
    NotFound = 1,
    Error = 2,
}
