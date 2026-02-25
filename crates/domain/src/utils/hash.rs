use crate::model::commit::{FileEntry, NewCommit};
use crate::model::errors::CommitError;
use serde_json;
use sha2::{Digest, Sha256};

/// Deterministic bincode config for file entries (same bytes for same Vec<FileEntry>).
fn bincode_config() -> bincode::config::Configuration {
    bincode::config::standard()
}

/// Compute SHA-256 hash of the bincode-serialized file entries.
/// Returns 64-char hex. Used as content-addressable ref for the files list object.
pub fn hash_file_entries(entries: &[FileEntry]) -> Result<String, CommitError> {
    let bytes = bincode::serde::encode_to_vec(entries, bincode_config())
        .map_err(|e| CommitError::CreationError(e.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn hash_commit(commit: &NewCommit) -> Result<String, CommitError> {
    // Create a structured representation of the commit data
    // This ensures consistent ordering of fields and includes all mandatory data
    let commit_data = serde_json::to_string(&commit)?;

    // Hash the serialized commit data
    let mut hasher = Sha256::new();
    hasher.update(commit_data.as_bytes());
    let hash = hasher.finalize();

    Ok(format!("{:x}", hash))
}

/// Compute a SHA-256 hash that identifies a snapshot.
///
/// The hash is derived from the source path and the timestamp at which the
/// snapshot was initiated, producing a unique 64-char hex identifier per
/// (source, time) pair without touching the filesystem.
pub fn hash_snapshot(source_path: &str, timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    let data = format!("{}|{}", source_path, timestamp.to_rfc3339());
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn hash_snapshot_is_deterministic() {
        let ts = chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let h1 = hash_snapshot("/data/main", &ts);
        let h2 = hash_snapshot("/data/main", &ts);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
        assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hash_snapshot_differs_on_different_timestamp() {
        let ts1 = chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let ts2 = chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 1).unwrap();
        assert_ne!(
            hash_snapshot("/data/main", &ts1),
            hash_snapshot("/data/main", &ts2)
        );
    }

    #[test]
    fn hash_snapshot_differs_on_different_source() {
        let ts = chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        assert_ne!(
            hash_snapshot("/data/main", &ts),
            hash_snapshot("/data/other", &ts)
        );
    }
}
