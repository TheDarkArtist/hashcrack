use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::error::{AppError, Result};

/// Serializable checkpoint state for resume capability.
#[derive(Deserialize, Debug)]
pub struct Checkpoint {
    pub line_offset: usize,
}

/// Load a checkpoint from disk.
pub fn load(path: &Path) -> Result<Checkpoint> {
    let data = fs::read_to_string(path).map_err(|e| AppError::Checkpoint {
        reason: format!("failed to read {}: {e}", path.display()),
    })?;
    serde_json::from_str(&data).map_err(|e| AppError::Checkpoint {
        reason: format!("invalid checkpoint format: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_roundtrip() {
        let json = r#"{"line_offset": 42}"#;
        let loaded: Checkpoint = serde_json::from_str(json).unwrap();
        assert_eq!(loaded.line_offset, 42);
    }

    #[test]
    fn checkpoint_ignores_extra_fields() {
        let json = r#"{"line_offset": 10, "algorithm": "sha1", "extra": true}"#;
        let loaded: Checkpoint = serde_json::from_str(json).unwrap();
        assert_eq!(loaded.line_offset, 10);
    }
}
