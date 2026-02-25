use std::fmt;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::model::errors::RepoError;
use crate::model::layout::{CONFIG_FILE, GFS_DIR};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub database_provider: String,
    pub database_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Runtime backend (e.g. `"docker"`, `"firecracker"`).
    pub runtime_provider: String,
    pub runtime_version: String,
    pub container_name: String,
}

impl fmt::Display for RuntimeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RuntimeConfig(provider: {}, version: {}, container: {})",
            self.runtime_provider, self.runtime_version, self.container_name
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GfsConfig {
    pub mount_point: Option<String>,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub user: Option<UserConfig>,
    #[serde(default)]
    pub environment: Option<EnvironmentConfig>,
    #[serde(default)]
    pub runtime: Option<RuntimeConfig>,
}

impl GfsConfig {
    pub fn load(repo_path: &Path) -> Result<Self, RepoError> {
        let config_path = repo_path.join(GFS_DIR).join(CONFIG_FILE);
        let content = std::fs::read_to_string(config_path)?;
        let config =
            toml::from_str(&content).map_err(|e| RepoError::InvalidConfig(e.to_string()))?;
        Ok(config)
    }

    pub fn save(&self, repo_path: &Path) -> Result<(), RepoError> {
        let config_path = repo_path.join(GFS_DIR).join(CONFIG_FILE);
        let content =
            toml::to_string_pretty(self).map_err(|e| RepoError::InvalidConfig(e.to_string()))?;
        std::fs::write(config_path, content)?;
        Ok(())
    }
}
