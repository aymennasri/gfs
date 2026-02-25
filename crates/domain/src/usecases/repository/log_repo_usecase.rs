use std::path::PathBuf;
use std::sync::Arc;

use thiserror::Error;

use crate::model::commit::CommitWithRefs;
use crate::ports::repository::{LogOptions, Repository, RepositoryError};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum LogRepoError {
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),
}

// ---------------------------------------------------------------------------
// Use case
// ---------------------------------------------------------------------------

/// Use case for displaying commit history reachable from HEAD (or a given revision).
///
/// Delegates to the [`Repository::log`] port. No direct I/O; all I/O is in the adapter.
pub struct LogRepoUseCase {
    repository: Arc<dyn Repository>,
}

impl LogRepoUseCase {
    pub fn new(repository: Arc<dyn Repository>) -> Self {
        Self { repository }
    }

    /// Return the commit history at `path` according to `options`.
    pub async fn run(
        &self,
        path: PathBuf,
        options: LogOptions,
    ) -> std::result::Result<Vec<CommitWithRefs>, LogRepoError> {
        self.repository
            .log(&path, options)
            .await
            .map_err(LogRepoError::from)
    }
}
