use gfs_domain::ports::compute::ComputeError;

/// Classify a bollard error into the appropriate [`ComputeError`] variant.
///
/// Bollard surfaces Docker daemon errors as [`bollard::errors::Error::DockerResponseServerError`]
/// with an HTTP status code and a message string. We inspect both the status and
/// the message body to produce the most specific `ComputeError`.
pub(crate) fn classify(container_id: &str, err: bollard::errors::Error) -> ComputeError {
    match &err {
        bollard::errors::Error::DockerResponseServerError {
            status_code,
            message,
        } => {
            let msg = message.to_ascii_lowercase();
            match status_code {
                404 => ComputeError::NotFound(if container_id.is_empty() {
                    message.clone()
                } else {
                    container_id.to_owned()
                }),
                409 => {
                    if msg.contains("already started")
                        || msg.contains("is already running")
                        || msg.contains("container already running")
                    {
                        ComputeError::AlreadyRunning(container_id.to_owned())
                    } else if msg.contains("is not running") || msg.contains("not running") {
                        ComputeError::NotRunning(container_id.to_owned())
                    } else if msg.contains("already paused") {
                        ComputeError::AlreadyPaused(container_id.to_owned())
                    } else if msg.contains("is not paused") || msg.contains("not paused") {
                        ComputeError::NotPaused(container_id.to_owned())
                    } else {
                        ComputeError::Internal(message.clone())
                    }
                }
                _ => ComputeError::Internal(message.clone()),
            }
        }
        bollard::errors::Error::IOError { err } => ComputeError::Internal(err.to_string()),
        other => ComputeError::Internal(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    fn docker_err(status_code: u16, message: impl Into<String>) -> bollard::errors::Error {
        bollard::errors::Error::DockerResponseServerError {
            status_code,
            message: message.into(),
        }
    }

    #[test]
    fn classify_404_with_container_id() {
        let err = classify("cid-123", docker_err(404, "not found"));
        assert!(matches!(err, ComputeError::NotFound(s) if s == "cid-123"));
    }

    #[test]
    fn classify_404_empty_container_id_uses_message() {
        let err = classify("", docker_err(404, "No such container"));
        assert!(matches!(err, ComputeError::NotFound(s) if s == "No such container"));
    }

    #[test]
    fn classify_409_already_running() {
        let err = classify("c1", docker_err(409, "Container is already running"));
        assert!(matches!(err, ComputeError::AlreadyRunning(s) if s == "c1"));
    }

    #[test]
    fn classify_409_not_running() {
        let err = classify("c1", docker_err(409, "Container is not running"));
        assert!(matches!(err, ComputeError::NotRunning(s) if s == "c1"));
    }

    #[test]
    fn classify_409_already_paused() {
        let err = classify("c1", docker_err(409, "Container already paused"));
        assert!(matches!(err, ComputeError::AlreadyPaused(s) if s == "c1"));
    }

    #[test]
    fn classify_409_not_paused() {
        let err = classify("c1", docker_err(409, "Container is not paused"));
        assert!(matches!(err, ComputeError::NotPaused(s) if s == "c1"));
    }

    #[test]
    fn classify_409_fallback_to_internal() {
        let err = classify("c1", docker_err(409, "Some other conflict"));
        assert!(matches!(err, ComputeError::Internal(_)));
    }

    #[test]
    fn classify_500_internal() {
        let err = classify("c1", docker_err(500, "Server error"));
        assert!(matches!(err, ComputeError::Internal(s) if s == "Server error"));
    }

    #[test]
    fn classify_io_error() {
        let err = classify(
            "c1",
            bollard::errors::Error::IOError {
                err: io::Error::new(io::ErrorKind::ConnectionRefused, "connection refused"),
            },
        );
        assert!(matches!(err, ComputeError::Internal(_)));
    }
}
