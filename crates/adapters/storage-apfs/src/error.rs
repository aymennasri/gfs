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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_diskutil_stderr_not_found() {
        let err = classify_diskutil_stderr("vol1", "Could not find disk");
        assert!(matches!(err, StorageError::NotFound(s) if s == "vol1"));

        let err = classify_diskutil_stderr("vol2", "no such volume");
        assert!(matches!(err, StorageError::NotFound(s) if s == "vol2"));
    }

    #[test]
    fn classify_diskutil_stderr_busy() {
        let err = classify_diskutil_stderr("vol1", "Volume is busy");
        assert!(matches!(err, StorageError::Busy(s) if s == "vol1"));
    }

    #[test]
    fn classify_diskutil_stderr_already_exists() {
        let err = classify_diskutil_stderr("vol1", "Volume already mounted");
        assert!(matches!(err, StorageError::AlreadyExists(s) if s == "vol1"));
    }

    #[test]
    fn classify_diskutil_stderr_internal() {
        let err = classify_diskutil_stderr("vol1", "Unknown diskutil error");
        assert!(matches!(err, StorageError::Internal(s) if s == "Unknown diskutil error"));
    }
}
