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
