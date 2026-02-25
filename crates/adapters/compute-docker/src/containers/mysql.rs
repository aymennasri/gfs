//! MySQL provider: compute definition, connection string, and related behaviour.

use std::path::PathBuf;
use std::sync::Arc;

use gfs_domain::ports::compute::{ComputeDefinition, EnvVar, PortMapping};
use gfs_domain::ports::database_provider::{
    ConnectionParams, DatabaseProvider, DatabaseProviderArg, DatabaseProviderRegistry,
    ProviderError, Result, SIGTERM, SupportedFeature,
};

const NAME: &str = "mysql";

/// Default MySQL image (official image).
const DEFAULT_IMAGE: &str = "mysql:latest";

/// Path inside the container where MySQL stores data.
const CONTAINER_DATA_DIR: &str = "/var/lib/mysql";

const ENV_ROOT_PASSWORD: &str = "MYSQL_ROOT_PASSWORD";
const ENV_DATABASE: &str = "MYSQL_DATABASE";

const DEFAULT_ROOT_PASSWORD: &str = "mysql";
const DEFAULT_DB: &str = "mysql";

/// MySQL compute definition provider. Supplies the definition and
/// provider-specific behaviour (connection string, name, default port).
#[derive(Debug)]
pub struct MysqlProvider;

impl MysqlProvider {
    pub fn new() -> Self {
        Self
    }

    fn definition_impl() -> ComputeDefinition {
        ComputeDefinition {
            image: DEFAULT_IMAGE.to_string(),
            env: vec![
                EnvVar {
                    name: ENV_ROOT_PASSWORD.to_string(),
                    default: Some(DEFAULT_ROOT_PASSWORD.to_string()),
                },
                EnvVar {
                    name: ENV_DATABASE.to_string(),
                    default: Some(DEFAULT_DB.to_string()),
                },
            ],
            ports: vec![PortMapping {
                compute_port: 3306,
                host_port: None,
            }],
            data_dir: PathBuf::from(CONTAINER_DATA_DIR),
            host_data_dir: None, // set by caller at provision time
            logs_dir: None,
            conf_dir: None,
            args: vec![],
        }
    }
}

impl Default for MysqlProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseProvider for MysqlProvider {
    fn name(&self) -> &str {
        NAME
    }

    fn definition(&self) -> ComputeDefinition {
        let mut def = Self::definition_impl();
        def.args = self
            .default_args()
            .into_iter()
            .flat_map(|a| {
                if a.value.is_empty() {
                    vec![a.name]
                } else {
                    vec![a.name, a.value]
                }
            })
            .collect();
        def
    }

    fn default_port(&self) -> u16 {
        3306
    }

    fn default_args(&self) -> Vec<DatabaseProviderArg> {
        vec![
            DatabaseProviderArg {
                name: "--skip-name-resolve".into(),
                value: String::new(),
            },
            DatabaseProviderArg {
                name: "--max_connections=5".into(),
                value: String::new(),
            },
            DatabaseProviderArg {
                name: "--max_connect_errors=100".into(),
                value: String::new(),
            },
            DatabaseProviderArg {
                name: "--table_open_cache=4".into(),
                value: String::new(),
            },
            DatabaseProviderArg {
                name: "--thread_cache_size=1".into(),
                value: String::new(),
            },
            DatabaseProviderArg {
                name: "--max_allowed_packet=256K".into(),
                value: String::new(),
            },
            DatabaseProviderArg {
                name: "--net_buffer_length=8K".into(),
                value: String::new(),
            },
            DatabaseProviderArg {
                name: "--innodb_buffer_pool_size=4M".into(),
                value: String::new(),
            },
            DatabaseProviderArg {
                name: "--innodb_log_buffer_size=128K".into(),
                value: String::new(),
            },
            DatabaseProviderArg {
                name: "--innodb_flush_method=O_DIRECT".into(),
                value: String::new(),
            },
            DatabaseProviderArg {
                name: "--innodb_flush_log_at_trx_commit=2".into(),
                value: String::new(),
            },
            DatabaseProviderArg {
                name: "--innodb_file_per_table=1".into(),
                value: String::new(),
            },
            DatabaseProviderArg {
                name: "--innodb_read_io_threads=1".into(),
                value: String::new(),
            },
            DatabaseProviderArg {
                name: "--innodb_write_io_threads=1".into(),
                value: String::new(),
            },
            DatabaseProviderArg {
                name: "--performance_schema=0".into(),
                value: String::new(),
            },
            DatabaseProviderArg {
                name: "--thread_stack=192K".into(),
                value: String::new(),
            },
            DatabaseProviderArg {
                name: "--skip-log-bin".into(),
                value: String::new(),
            },
            DatabaseProviderArg {
                name: "--bind-address=0.0.0.0".into(),
                value: String::new(),
            },
        ]
    }

    fn default_signal(&self) -> u32 {
        SIGTERM
    }

    fn connection_string(
        &self,
        params: &ConnectionParams,
    ) -> std::result::Result<String, ProviderError> {
        let user = "root";
        let password = params
            .get_env(ENV_ROOT_PASSWORD)
            .unwrap_or(DEFAULT_ROOT_PASSWORD);
        let db = params.get_env(ENV_DATABASE).unwrap_or(DEFAULT_DB);
        Ok(format!(
            "mysql://{}:{}@{}:{}/{}",
            user, password, params.host, params.port, db
        ))
    }

    fn supported_versions(&self) -> Vec<String> {
        vec!["8.0".into(), "8.1".into()]
    }

    fn supported_features(&self) -> Vec<SupportedFeature> {
        vec![
            SupportedFeature {
                id: "tls".into(),
                description: "TLS/SSL encryption for connections.".into(),
            },
            SupportedFeature {
                id: "schema".into(),
                description: "Schema and DDL management.".into(),
            },
            SupportedFeature {
                id: "masking".into(),
                description: "Data masking and redaction.".into(),
            },
            SupportedFeature {
                id: "backup".into(),
                description: "Backup and restore.".into(),
            },
            SupportedFeature {
                id: "import".into(),
                description: "Data import from external sources.".into(),
            },
        ]
    }

    fn prepare_for_snapshot(&self, _params: &ConnectionParams) -> Result<Vec<String>> {
        Ok(vec![])
    }
}

/// Registers the MySQL provider in `registry` under the name `"mysql"`.
pub fn register(registry: &impl DatabaseProviderRegistry) -> Result<()> {
    registry.register(Arc::new(MysqlProvider::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_string_uses_defaults() {
        let provider = MysqlProvider::new();
        let params = ConnectionParams {
            host: "localhost".to_string(),
            port: 3306,
            env: vec![],
        };
        let s = provider.connection_string(&params).unwrap();
        assert_eq!(s, "mysql://root:mysql@localhost:3306/mysql");
    }

    #[test]
    fn connection_string_uses_env_overrides() {
        let provider = MysqlProvider::new();
        let params = ConnectionParams {
            host: "db.example.com".to_string(),
            port: 13306,
            env: vec![
                ("MYSQL_ROOT_PASSWORD".to_string(), "secret".to_string()),
                ("MYSQL_DATABASE".to_string(), "mydb".to_string()),
            ],
        };
        let s = provider.connection_string(&params).unwrap();
        assert_eq!(s, "mysql://root:secret@db.example.com:13306/mydb");
    }

    #[test]
    fn name_and_default_port() {
        let provider = MysqlProvider::new();
        assert_eq!(provider.name(), "mysql");
        assert_eq!(provider.default_port(), 3306);
    }

    #[test]
    fn supported_versions_non_empty() {
        let provider = MysqlProvider::new();
        let versions = provider.supported_versions();
        assert!(!versions.is_empty());
        assert!(versions.contains(&"8.0".to_string()));
    }

    #[test]
    fn supported_features_contains_tls_and_schema() {
        let provider = MysqlProvider::new();
        let features = provider.supported_features();
        let ids: Vec<_> = features.iter().map(|f| f.id.as_str()).collect();
        assert!(ids.contains(&"tls"));
        assert!(ids.contains(&"schema"));
    }

    #[test]
    fn feature_description_returns_some_for_backup() {
        let provider = MysqlProvider::new();
        let desc = provider.feature_description("backup");
        assert!(desc.is_some());
        assert!(desc.unwrap().contains("Backup"));
    }

    #[test]
    fn default_signal_is_sigterm() {
        let provider = MysqlProvider::new();
        assert_eq!(provider.default_signal(), SIGTERM);
    }

    #[test]
    fn default_args_non_empty_and_definition_includes_them() {
        let provider = MysqlProvider::new();
        let args = provider.default_args();
        assert!(!args.is_empty());
        assert_eq!(
            args.first().map(|a| a.name.as_str()),
            Some("--skip-name-resolve")
        );
        let def = provider.definition();
        assert_eq!(def.args.len(), args.len());
        assert_eq!(def.args.first(), Some(&"--skip-name-resolve".to_string()));
        assert_eq!(def.args.last(), Some(&"--bind-address=0.0.0.0".to_string()));
    }
}
