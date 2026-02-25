use gfs_domain::ports::storage::StorageError;

/// Classify a `diskutil` stderr message into the appropriate [`StorageError`] variant.
pub(crate) fn classify_diskutil_stderr(volume_id: &str, stderr: &str) -> StorageError {
    let lower = stderr.to_lowercase();
    if lower.contains("could not find") || lower.contains("no such") || lower.contains("not find") {
        StorageError::NotFound(volume_id.to_owned())
    } else if lower.contains("busy") {
        StorageError::Busy(volume_id.to_owned())
    } else if lower.contains("already") {
        StorageError::AlreadyExists(volume_id.to_owned())
    } else {
        StorageError::Internal(stderr.trim().to_owned())
    }
}
