use std::path::PathBuf;
use std::sync::Arc;

use thiserror::Error;

use crate::model::config::{EnvironmentConfig, RuntimeConfig};
use crate::ports::compute::{Compute, ComputeError, StartOptions};
use crate::ports::database_provider::DatabaseProviderRegistry;
use crate::ports::repository::{Repository, RepositoryError};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum InitRepoError {
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("compute error: {0}")]
    Compute(#[from] ComputeError),

    #[error("unknown database provider: '{0}'")]
    UnknownDatabaseProvider(String),

    #[error("database_version is required when database_provider is set")]
    DatabaseVersionRequired,
}

// ---------------------------------------------------------------------------
// Use case
// ---------------------------------------------------------------------------

/// Use case for initialising a repository and optionally provisioning a database.
///
/// `R` is generic over [`DatabaseProviderRegistry`] because that trait is not
/// dyn-compatible (its `register` method uses `impl Into<String>`).
pub struct InitRepositoryUseCase<R: DatabaseProviderRegistry> {
    repository: Arc<dyn Repository>,
    compute: Arc<dyn Compute>,
    registry: Arc<R>,
}

impl<R: DatabaseProviderRegistry> InitRepositoryUseCase<R> {
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

    /// Initialise the repository and optionally provision a database.
    ///
    /// When `database_provider` is set, `database_version` must also be set and non-empty.
    pub async fn run(
        &self,
        path: PathBuf,
        mount_point: Option<String>,
        database_provider: Option<String>,
        database_version: Option<String>,
    ) -> std::result::Result<(), InitRepoError> {
        self.repository.init(&path, mount_point).await?;

        if let Some(provider) = database_provider {
            let version = database_version
                .filter(|v| !v.is_empty())
                .ok_or(InitRepoError::DatabaseVersionRequired)?;
            self.deploy_database(&path, provider, version).await?;
        }

        Ok(())
    }

    async fn deploy_database(
        &self,
        repo_path: &std::path::Path,
        provider_name: String,
        database_version: String,
    ) -> std::result::Result<(), InitRepoError> {
        let list = self.registry.list();
        let matched_name = list
            .iter()
            .find(|n| n.eq_ignore_ascii_case(&provider_name))
            .cloned();

        let provider = matched_name
            .and_then(|name| self.registry.get(&name))
            .ok_or_else(|| {
                InitRepoError::UnknownDatabaseProvider(format!(
                    "'{}'; available: {}",
                    provider_name,
                    list.join(", ")
                ))
            })?;

        let mut definition = provider.definition();
        let base = definition
            .image
            .split(':')
            .next()
            .unwrap_or(&definition.image);
        definition.image = format!("{}:{}", base, database_version);

        let workspace_data_dir = self
            .repository
            .get_workspace_data_dir_for_head(repo_path)
            .await?;
        definition.host_data_dir = Some(workspace_data_dir);

        let id = self.compute.provision(&definition).await?;
        self.compute.start(&id, StartOptions::default()).await?;

        let database_version = provider.version_from_image(&definition);

        let environment = EnvironmentConfig {
            database_provider: provider_name,
            database_version,
        };
        self.repository
            .update_environment_config(repo_path, environment)
            .await?;

        let runtime = RuntimeConfig {
            runtime_provider: "docker".to_string(),
            runtime_version: "24".to_string(),
            container_name: id.0.clone(),
        };
        self.repository
            .update_runtime_config(repo_path, runtime)
            .await?;

        tracing::info!("Database deployed; instance id: {}", id);
        Ok(())
    }
}
