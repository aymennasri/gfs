//! Use case for switching the active branch or commit (checkout).
//!
//! Orchestrates [`Repository`], [`Compute`], and [`DatabaseProviderRegistry`]:
//! stops the repo's compute instance (if any), runs checkout, then starts or
//! recreates the instance with a mount on the new branch/commit data dir.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use thiserror::Error;

use crate::model::config::RuntimeConfig;
use crate::ports::compute::{Compute, ComputeError, InstanceId};
use crate::ports::database_provider::DatabaseProviderRegistry;
use crate::ports::repository::{Repository, RepositoryError};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum CheckoutRepoError {
    #[error("{0}")]
    Repository(#[from] RepositoryError),

    #[error("compute: {0}")]
    Compute(#[from] ComputeError),
}

// ---------------------------------------------------------------------------
// Use case
// ---------------------------------------------------------------------------

/// Use case for checking out a branch or commit.
///
/// When the repo has a compute container configured, stops it before checkout
/// and starts (or recreates with the new workspace mount) after checkout.
/// Resolves the revision, runs checkout, and returns the full commit hash.
pub struct CheckoutRepoUseCase<R: DatabaseProviderRegistry> {
    repository: Arc<dyn Repository>,
    compute: Arc<dyn Compute>,
    registry: Arc<R>,
}

impl<R: DatabaseProviderRegistry> CheckoutRepoUseCase<R> {
    pub fn new(
        repository: Arc<dyn Repository>,
        compute: Arc<dyn Compute>,
        registry: Arc<R>,
    ) -> Self {
        Self {
            repository,
            compute,
            registry,
        }
    }

    /// Check out `revision` (branch name or full 64-char commit hash) at `path`.
    /// When `create_branch` is `Some(name)`, creates a new branch at `revision`
    /// (or current HEAD if `revision` is empty) then checks out that branch.
    /// Returns the full commit hash on success for display (e.g. short hash).
    pub async fn run(
        &self,
        path: PathBuf,
        revision: String,
        create_branch: Option<String>,
    ) -> std::result::Result<String, CheckoutRepoError> {
        let revision = revision.trim().to_string();

        let container_id = self
            .repository
            .get_runtime_config(&path)
            .await
            .ok()
            .flatten()
            .and_then(|r| {
                let name = r.container_name.trim();
                if name.is_empty() {
                    None
                } else {
                    Some(InstanceId(name.to_string()))
                }
            });

        if let Some(ref id) = container_id {
            let _ = self.compute.stop(id).await?;
        }

        let commit_hash = self.do_checkout(&path, &revision, create_branch).await?;

        if let Some(ref id) = container_id {
            self.ensure_compute_started_after_checkout(&path, id)
                .await?;
        }

        Ok(commit_hash)
    }

    async fn do_checkout(
        &self,
        path: &Path,
        revision: &str,
        create_branch: Option<String>,
    ) -> std::result::Result<String, CheckoutRepoError> {
        if let Some(ref branch_name) = create_branch {
            let branch_name = branch_name.trim().to_string();
            if branch_name.is_empty() {
                return Err(CheckoutRepoError::Repository(
                    RepositoryError::RevisionNotFound("(empty branch name)".to_string()),
                ));
            }
            let start_rev = if revision.is_empty() {
                "HEAD".to_string()
            } else {
                revision.to_string()
            };
            let commit_hash = self.repository.rev_parse(path, &start_rev).await?;
            if commit_hash == "0" {
                return Err(CheckoutRepoError::Repository(RepositoryError::Internal(
                    "cannot create branch: start revision has no commits".to_string(),
                )));
            }
            self.repository
                .create_branch(path, &branch_name, &commit_hash)
                .await?;
            self.repository.checkout(path, &branch_name).await?;
            let out_hash = self.repository.get_current_commit_id(path).await?;
            return Ok(out_hash);
        }

        if revision.is_empty() {
            return Err(CheckoutRepoError::Repository(
                RepositoryError::RevisionNotFound("(empty)".to_string()),
            ));
        }

        self.repository.checkout(path, revision).await?;
        let commit_hash = self.repository.get_current_commit_id(path).await?;
        Ok(commit_hash)
    }

    /// Start the instance or recreate it with the current workspace data dir if the bind differs.
    async fn ensure_compute_started_after_checkout(
        &self,
        path: &Path,
        instance_id: &InstanceId,
    ) -> std::result::Result<(), CheckoutRepoError> {
        let active = self.repository.get_active_workspace_data_dir(path).await?;
        let active_str = active.to_string_lossy().into_owned();
        tracing::info!(
            "ensure_compute_started_after_checkout: active_workspace={:?}",
            active
        );

        let environment = match self.repository.get_environment_config(path).await? {
            Some(e) if !e.database_provider.is_empty() => e,
            _ => return Ok(()),
        };

        let provider = match self.registry.get(environment.database_provider.as_str()) {
            Some(p) => p,
            None => return Ok(()),
        };

        let mut definition = provider.definition();
        if !environment.database_version.is_empty() {
            let base = definition
                .image
                .split(':')
                .next()
                .unwrap_or(definition.image.as_str());
            definition.image = format!("{}:{}", base, environment.database_version);
        }
        definition.host_data_dir = Some(active.clone());
        let compute_data_path = definition.data_dir.to_string_lossy().into_owned();

        let current_bind = self
            .compute
            .get_instance_data_mount_host_path(instance_id, &compute_data_path)
            .await
            .ok()
            .flatten()
            .map(|p| p.to_string_lossy().into_owned());

        tracing::info!(
            "ensure_compute_started_after_checkout: current_bind={:?}, paths_differ={}",
            current_bind,
            paths_differ(&active_str, current_bind.as_deref().unwrap_or(""))
        );

        if !paths_differ(&active_str, current_bind.as_deref().unwrap_or("")) {
            tracing::info!("ensure_compute_started_after_checkout: starting existing container");
            let _ = self.compute.start(instance_id, Default::default()).await?;
            return Ok(());
        }

        tracing::info!(
            "ensure_compute_started_after_checkout: removing old container and creating new one"
        );
        self.compute.remove_instance(instance_id).await?;
        let new_id = self.compute.provision(&definition).await?;
        let _ = self.compute.start(&new_id, Default::default()).await?;
        self.repository
            .update_runtime_config(
                path,
                RuntimeConfig {
                    runtime_provider: "docker".to_string(),
                    runtime_version: "24".to_string(),
                    container_name: new_id.0.clone(),
                },
            )
            .await?;
        Ok(())
    }
}

fn paths_differ(active: &str, current_bind: &str) -> bool {
    let a = std::path::Path::new(active);
    let b = std::path::Path::new(current_bind);
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a != b,
        _ => active != current_bind,
    }
}
