//! Registry of database **providers**. Each provider supplies a
//! [`ComputeDefinition`] and provider-specific behaviour (connection string,
//! name, version extraction, etc.).
//!
//! Use [`DatabaseProviderRegistry::register`] to add a provider, and
//! [`DatabaseProviderRegistry::get`] / [`DatabaseProviderRegistry::list`] to look them up.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::ports::compute::ComputeDefinition;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("definition already registered: '{0}'")]
    AlreadyRegistered(String),

    #[error("definition not found: '{0}'")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("missing required env var for connection string: '{0}'")]
    MissingEnvVar(String),

    #[error("invalid connection params: {0}")]
    InvalidParams(String),
}

pub type Result<T> = std::result::Result<T, RegistryError>;

// ---------------------------------------------------------------------------
// Connection params
// ---------------------------------------------------------------------------

/// Parameters used by a provider to build a client connection string.
/// `env` typically holds container environment (e.g. POSTGRES_USER, POSTGRES_PASSWORD).
#[derive(Debug, Clone, Default)]
pub struct ConnectionParams {
    pub host: String,
    pub port: u16,
    /// Environment variables (e.g. from the running container) for user, password, db name.
    pub env: Vec<(String, String)>,
}

impl ConnectionParams {
    /// Look up an env var by name (case-sensitive).
    pub fn get_env(&self, name: &str) -> Option<&str> {
        self.env
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

// ---------------------------------------------------------------------------
// Supported feature
// ---------------------------------------------------------------------------

/// A supported feature with an identifier and human-readable description.
/// Used for discovery/listing (e.g. `gfs providers`).
#[derive(Debug, Clone)]
pub struct SupportedFeature {
    /// Feature identifier (e.g. `"tls"`, `"schema"`, `"backup"`).
    pub id: String,
    /// Short human-readable description of the feature.
    pub description: String,
}

/// A database provider argument.
#[derive(Debug, Clone)]
pub struct DatabaseProviderArg {
    /// Argument name (e.g. `"tls"`, `"schema"`, `"backup"`).
    pub name: String,
    /// Argument value.
    pub value: String,
}

/// Signal number for graceful shutdown. On Unix, 15 = SIGTERM.
pub const SIGTERM: u32 = 15;

// ---------------------------------------------------------------------------
// Provider port
// ---------------------------------------------------------------------------

/// A database provider: supplies a definition and provider-specific behaviour.
/// Implementations (e.g. postgresql, mysql) are registered in a [`DatabaseProviderRegistry`].
pub trait DatabaseProvider: Send + Sync {
    /// Display name used to register and look up this provider (e.g. `"postgresql"`).
    fn name(&self) -> &str;

    /// Compute definition used for provisioning (image, env, ports, data dir, etc.).
    fn definition(&self) -> ComputeDefinition;

    /// Default container port for this database (e.g. 5432 for PostgreSQL).
    fn default_port(&self) -> u16;

    /// Default arguments for this database provider.
    fn default_args(&self) -> Vec<DatabaseProviderArg>;

    /// Default signal sent to the database process when stopping (e.g. for graceful shutdown).
    /// Returns the signal number (e.g. [`SIGTERM`] = 15 on Unix). Default implementation returns SIGTERM.
    fn default_signal(&self) -> u32 {
        SIGTERM
    }

    /// Build a client connection string from host, port, and optional env (credentials, db name).
    fn connection_string(
        &self,
        params: &ConnectionParams,
    ) -> std::result::Result<String, ProviderError>;

    /// Extract version string from the definition's image (e.g. `postgres:16` → `"16"`).
    fn version_from_image(&self, definition: &ComputeDefinition) -> String {
        definition
            .image
            .split(':')
            .nth(1)
            .unwrap_or("latest")
            .to_string()
    }

    /// List of supported version tags (e.g. `"16"`, `"8.0"`). Used for discovery/listing (e.g. `gfs providers`).
    fn supported_versions(&self) -> Vec<String>;

    /// List of supported features with id and description. Used for discovery/listing (e.g. `gfs providers`).
    fn supported_features(&self) -> Vec<SupportedFeature>;

    /// Return the description for a feature by id. Returns `None` if the feature is not supported.
    fn feature_description(&self, feature_id: &str) -> Option<String> {
        self.supported_features()
            .into_iter()
            .find(|f| f.id == feature_id)
            .map(|f| f.description)
    }

    /// Prepare the database provider for snapshotting.
    /// Returns a list of commands to run before taking the snapshot (e.g. `psql -U user -c "CHECKPOINT;"`).
    /// The compute runtime runs these commands in the container before taking the snapshot.
    fn prepare_for_snapshot(&self, params: &ConnectionParams) -> Result<Vec<String>>;
}

// ---------------------------------------------------------------------------
// Registry port
// ---------------------------------------------------------------------------

/// Port for a registry of database **providers**. Callers register
/// provider implementations and look them up by name for provisioning and
/// provider-specific operations (e.g. connection string).
pub trait DatabaseProviderRegistry: Send + Sync {
    /// Register a provider. Overwrites any existing entry with the same name.
    fn register(&self, provider: Arc<dyn DatabaseProvider>) -> Result<()>;

    /// Return the provider for `name`, if registered.
    fn get(&self, name: &str) -> Option<Arc<dyn DatabaseProvider>>;

    /// Return the definition for `name`, if registered. Convenience over `get(name).map(|p| p.definition())`.
    fn get_definition(&self, name: &str) -> Option<ComputeDefinition> {
        self.get(name).map(|p| p.definition())
    }

    /// Return all registered provider names.
    fn list(&self) -> Vec<String>;

    /// Remove the provider for `name`. Returns the removed provider if it existed.
    fn unregister(&self, name: &str) -> Option<Arc<dyn DatabaseProvider>>;
}

// ---------------------------------------------------------------------------
// In-memory implementation
// ---------------------------------------------------------------------------

/// Default in-memory registry. Safe to share via `Arc<InMemoryDatabaseProviderRegistry>`.
#[derive(Default)]
pub struct InMemoryDatabaseProviderRegistry {
    providers: RwLock<HashMap<String, Arc<dyn DatabaseProvider>>>,
}

impl InMemoryDatabaseProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DatabaseProviderRegistry for InMemoryDatabaseProviderRegistry {
    fn register(&self, provider: Arc<dyn DatabaseProvider>) -> Result<()> {
        let name = provider.name().to_string();
        self.providers
            .write()
            .map_err(|_| RegistryError::Internal("lock poisoned".to_string()))?
            .insert(name, provider);
        Ok(())
    }

    fn get(&self, name: &str) -> Option<Arc<dyn DatabaseProvider>> {
        self.providers.read().ok()?.get(name).cloned()
    }

    fn list(&self) -> Vec<String> {
        self.providers
            .read()
            .map(|g| g.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default()
    }

    fn unregister(&self, name: &str) -> Option<Arc<dyn DatabaseProvider>> {
        self.providers.write().ok()?.remove(name)
    }
}
