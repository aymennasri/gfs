//! Use case for reporting data-plane status (RFC 006).
//!
//! Aggregates repository (current branch), config (provider, version), and compute
//! runtime (container status, id, connection string) into a read-only status response.
//! Connection string is built via [`DatabaseProvider::connection_string`] using
//! params from [`Compute::get_connection_info`].

use std::path::Path;

use crate::model::status::{ComputeStatus, StatusResponse};
use crate::ports::compute::{Compute, InstanceId};
use crate::ports::database_provider::{ConnectionParams, DatabaseProviderRegistry};
use crate::ports::repository::{Repository, RepositoryError};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum StatusRepoError {
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),
}

// ---------------------------------------------------------------------------
// Use case
// ---------------------------------------------------------------------------

/// Use case for reporting the current status of a GFS repository and its compute instance.
///
/// Steps:
/// 1. Resolve current branch from the repository.
/// 2. Load environment and runtime config from the repo.
/// 3. If runtime is configured, call Compute::status and build connection string via DatabaseProvider.
/// 4. Aggregate into [`StatusResponse`].
pub struct StatusRepoUseCase<R: DatabaseProviderRegistry> {
    repository: std::sync::Arc<dyn Repository>,
    compute: std::sync::Arc<dyn Compute>,
    registry: std::sync::Arc<R>,
}

impl<R: DatabaseProviderRegistry> StatusRepoUseCase<R> {
    pub fn new(
        repository: std::sync::Arc<dyn Repository>,
        compute: std::sync::Arc<dyn Compute>,
        registry: std::sync::Arc<R>,
    ) -> Self {
        Self {
            repository,
            compute,
            registry,
        }
    }

    /// Build the status response for the repository at `path`.
    ///
    /// Caller must ensure `path` is a valid GFS repo (e.g. resolve from CWD or `--path`).
    pub async fn run(&self, path: &Path) -> Result<StatusResponse, StatusRepoError> {
        let current_branch = self.repository.get_current_branch(path).await?;
        let environment = self.repository.get_environment_config(path).await?;
        let runtime = self.repository.get_runtime_config(path).await?;
        let active_workspace_data_dir = self
            .repository
            .get_active_workspace_data_dir(path)
            .await
            .ok()
            .map(|p| p.to_string_lossy().into_owned());

        let (compute, bind_mismatch_warning) = build_compute_status(
            &*self.compute,
            self.registry.as_ref(),
            environment.as_ref(),
            runtime.as_ref(),
            active_workspace_data_dir.as_deref(),
        )
        .await;

        Ok(StatusResponse {
            current_branch,
            compute,
            active_workspace_data_dir,
            bind_mismatch_warning,
        })
    }
}

/// Build compute status when environment and runtime config are present.
/// Returns (compute_status, bind_mismatch_warning) when the container is bound to a different path than the active workspace.
async fn build_compute_status<R: DatabaseProviderRegistry>(
    compute: &dyn Compute,
    registry: &R,
    environment: Option<&crate::model::config::EnvironmentConfig>,
    runtime: Option<&crate::model::config::RuntimeConfig>,
    active_workspace_data_dir: Option<&str>,
) -> (Option<ComputeStatus>, Option<String>) {
    let (env, runtime) = match (environment, runtime) {
        (Some(e), Some(r)) if !e.database_provider.is_empty() && !r.container_name.is_empty() => {
            (e, r)
        }
        _ => return (None, None),
    };

    let instance_id = InstanceId(runtime.container_name.clone());
    let provider_name = env.database_provider.as_str();
    let version = env.database_version.clone();

    let (container_status, container_id, connection_string, data_bind_host_path) =
        match compute.status(&instance_id).await {
            Ok(status) => {
                let container_id = status.id.0.clone();
                let container_status = status.state.as_status_str().to_string();
                let conn =
                    build_connection_string(compute, registry, &instance_id, provider_name).await;
                let data_bind =
                    get_data_bind_host_path(compute, registry, &instance_id, provider_name).await;
                (container_status, container_id, conn, data_bind)
            }
            Err(_) => (
                "not_provisioned".to_string(),
                runtime.container_name.clone(),
                String::new(),
                None,
            ),
        };

    let bind_mismatch_warning = match (active_workspace_data_dir, &data_bind_host_path) {
        (Some(active), Some(bind)) if paths_differ(active, bind) => Some(format!(
            "Container is bound to a different branch's data: {} (current branch uses {}). \
             Stop and start the container to use the current branch's data.",
            bind, active
        )),
        _ => None,
    };

    let compute_status = Some(ComputeStatus {
        provider: provider_name.to_string(),
        version,
        container_status,
        container_id,
        connection_string,
        data_bind_host_path,
    });

    (compute_status, bind_mismatch_warning)
}

fn paths_differ(a: &str, b: &str) -> bool {
    let a = std::path::Path::new(a);
    let b = std::path::Path::new(b);
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a != b,
        _ => a != b,
    }
}

async fn get_data_bind_host_path<R: DatabaseProviderRegistry>(
    compute: &dyn Compute,
    registry: &R,
    instance_id: &InstanceId,
    provider_name: &str,
) -> Option<String> {
    let provider = registry.get(provider_name)?;
    let compute_data_path = provider
        .definition()
        .data_dir
        .to_string_lossy()
        .into_owned();
    compute
        .get_instance_data_mount_host_path(instance_id, &compute_data_path)
        .await
        .ok()
        .flatten()
        .map(|p| p.to_string_lossy().into_owned())
}

/// Build connection string using DatabaseProvider from the registry and connection info from Compute.
async fn build_connection_string<R: DatabaseProviderRegistry>(
    compute: &dyn Compute,
    registry: &R,
    instance_id: &InstanceId,
    provider_name: &str,
) -> String {
    let provider = match registry.get(provider_name) {
        Some(p) => p,
        None => return String::new(),
    };
    let compute_port = provider.default_port();
    let info = match compute.get_connection_info(instance_id, compute_port).await {
        Ok(i) => i,
        Err(_) => return String::new(),
    };
    let params = ConnectionParams {
        host: info.host,
        port: info.port,
        env: info.env,
    };
    provider.connection_string(&params).unwrap_or_default()
}
