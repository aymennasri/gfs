//! PostgreSQL provider: compute definition, connection string, and related behaviour.

use std::path::PathBuf;
use std::sync::Arc;

use gfs_domain::ports::compute::{ComputeDefinition, EnvVar, PortMapping};
use gfs_domain::ports::database_provider::{
    ConnectionParams, DatabaseProvider, DatabaseProviderArg, DatabaseProviderRegistry,
    ProviderError, Result, SIGTERM, SupportedFeature,
};

const NAME: &str = "postgres";

/// Default PostgreSQL image (official image, current LTS-alpine).
const DEFAULT_IMAGE: &str = "postgres:latest";

/// Path inside the container where PostgreSQL stores data (PGDATA).
const CONTAINER_DATA_DIR: &str = "/var/lib/postgresql/data";

const ENV_USER: &str = "POSTGRES_USER";
const ENV_PASSWORD: &str = "POSTGRES_PASSWORD";
const ENV_DB: &str = "POSTGRES_DB";

const DEFAULT_USER: &str = "postgres";
const DEFAULT_PASSWORD: &str = "postgres";
const DEFAULT_DB: &str = "postgres";

/// PostgreSQL compute definition provider. Supplies the definition and
/// provider-specific behaviour (connection string, name, default port).
#[derive(Debug)]
pub struct PostgresqlProvider;

impl PostgresqlProvider {
    pub fn new() -> Self {
        Self
    }

    fn definition_impl() -> ComputeDefinition {
        ComputeDefinition {
            image: DEFAULT_IMAGE.to_string(),
            env: vec![
                EnvVar {
                    name: ENV_USER.to_string(),
                    default: Some(DEFAULT_USER.to_string()),
                },
                EnvVar {
                    name: ENV_PASSWORD.to_string(),
                    default: Some(DEFAULT_PASSWORD.to_string()),
                },
                EnvVar {
                    name: ENV_DB.to_string(),
                    default: Some(DEFAULT_DB.to_string()),
                },
            ],
            ports: vec![PortMapping {
                compute_port: 5432,
                host_port: None,
            }],
            data_dir: PathBuf::from(CONTAINER_DATA_DIR),
            host_data_dir: None, // set by caller at provision time
            logs_dir: None,
            conf_dir: None,
            args: vec![],
        }
    }

    fn default_args_impl() -> Vec<DatabaseProviderArg> {
        vec![
            DatabaseProviderArg {
                name: "-c".into(),
                value: "shared_buffers=32MB".into(),
            },
            DatabaseProviderArg {
                name: "-c".into(),
                value: "work_mem=2MB".into(),
            },
            DatabaseProviderArg {
                name: "-c".into(),
                value: "maintenance_work_mem=4MB".into(),
            },
            DatabaseProviderArg {
                name: "-c".into(),
                value: "wal_buffers=4MB".into(),
            },
            DatabaseProviderArg {
                name: "-c".into(),
                value: "max_wal_size=128MB".into(),
            },
            DatabaseProviderArg {
                name: "-c".into(),
                value: "checkpoint_timeout=15min".into(),
            },
            DatabaseProviderArg {
                name: "-c".into(),
                value: "checkpoint_completion_target=0.9".into(),
            },
            DatabaseProviderArg {
                name: "-c".into(),
                value: "synchronous_commit=on".into(),
            },
            DatabaseProviderArg {
                name: "-c".into(),
                value: "max_connections=10".into(),
            },
            DatabaseProviderArg {
                name: "-c".into(),
                value: "max_parallel_workers=0".into(),
            },
            DatabaseProviderArg {
                name: "-c".into(),
                value: "max_parallel_workers_per_gather=0".into(),
            },
            DatabaseProviderArg {
                name: "-c".into(),
                value: "idle_in_transaction_session_timeout=60s".into(),
            },
            DatabaseProviderArg {
                name: "-c".into(),
                value: "log_min_duration_statement=1000".into(),
            },
            DatabaseProviderArg {
                name: "-c".into(),
                value: "autovacuum=on".into(),
            },
            DatabaseProviderArg {
                name: "-c".into(),
                value: "full_page_writes=on".into(),
            },
        ]
    }
}

impl Default for PostgresqlProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseProvider for PostgresqlProvider {
    fn name(&self) -> &str {
        NAME
    }

    fn definition(&self) -> ComputeDefinition {
        let mut def = Self::definition_impl();
        def.args = self
            .default_args()
            .into_iter()
            .flat_map(|a| [a.name, a.value])
            .collect();
        def
    }

    fn default_port(&self) -> u16 {
        5432
    }

    fn default_args(&self) -> Vec<DatabaseProviderArg> {
        Self::default_args_impl()
    }

    fn default_signal(&self) -> u32 {
        SIGTERM
    }

    fn connection_string(
        &self,
        params: &ConnectionParams,
    ) -> std::result::Result<String, ProviderError> {
        let user = params.get_env(ENV_USER).unwrap_or(DEFAULT_USER);
        let password = params.get_env(ENV_PASSWORD).unwrap_or(DEFAULT_PASSWORD);
        let db = params.get_env(ENV_DB).unwrap_or(DEFAULT_DB);
        Ok(format!(
            "postgresql://{}:{}@{}:{}/{}",
            user, password, params.host, params.port, db
        ))
    }

    fn supported_versions(&self) -> Vec<String> {
        vec![
            "13".into(),
            "14".into(),
            "15".into(),
            "16".into(),
            "17".into(),
            "18".into(),
        ]
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
                id: "auto-scaling".into(),
                description: "Automatic resource scaling.".into(),
            },
            SupportedFeature {
                id: "performance-profile".into(),
                description: "Performance tuning profiles.".into(),
            },
            SupportedFeature {
                id: "backup".into(),
                description: "Backup and restore.".into(),
            },
            SupportedFeature {
                id: "import".into(),
                description: "Data import from external sources.".into(),
            },
            SupportedFeature {
                id: "replication".into(),
                description: "Replication and high availability.".into(),
            },
            SupportedFeature {
                id: "ai-agents".into(),
                description: "AI agent integration.".into(),
            },
        ]
    }

    fn prepare_for_snapshot(&self, _params: &ConnectionParams) -> Result<Vec<String>> {
        // Use TCP (127.0.0.1) + env vars so the command works when run via docker exec as root.
        // Peer auth would fail for root; password auth over TCP works.
        Ok(vec![
            "PGPASSWORD=\"$POSTGRES_PASSWORD\" psql -h 127.0.0.1 -U \"$POSTGRES_USER\" -d \"$POSTGRES_DB\" -c \"CHECKPOINT;\""
                .to_string(),
        ])
    }
}

/// Registers the PostgreSQL provider in `registry` under the name `"postgres"`.
pub fn register(registry: &impl DatabaseProviderRegistry) -> Result<()> {
    registry.register(Arc::new(PostgresqlProvider::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_string_uses_defaults() {
        let provider = PostgresqlProvider::new();
        let params = ConnectionParams {
            host: "localhost".to_string(),
            port: 5432,
            env: vec![],
        };
        let s = provider.connection_string(&params).unwrap();
        assert_eq!(s, "postgresql://postgres:postgres@localhost:5432/postgres");
    }

    #[test]
    fn connection_string_uses_env_overrides() {
        let provider = PostgresqlProvider::new();
        let params = ConnectionParams {
            host: "db.example.com".to_string(),
            port: 15432,
            env: vec![
                ("POSTGRES_USER".to_string(), "myuser".to_string()),
                ("POSTGRES_PASSWORD".to_string(), "secret".to_string()),
                ("POSTGRES_DB".to_string(), "mydb".to_string()),
            ],
        };
        let s = provider.connection_string(&params).unwrap();
        assert_eq!(s, "postgresql://myuser:secret@db.example.com:15432/mydb");
    }

    #[test]
    fn name_and_default_port() {
        let provider = PostgresqlProvider::new();
        assert_eq!(provider.name(), "postgres");
        assert_eq!(provider.default_port(), 5432);
    }

    #[test]
    fn supported_versions_non_empty() {
        let provider = PostgresqlProvider::new();
        let versions = provider.supported_versions();
        assert!(!versions.is_empty());
        assert!(versions.contains(&"16".to_string()));
    }

    #[test]
    fn supported_features_contains_tls_and_schema() {
        let provider = PostgresqlProvider::new();
        let features = provider.supported_features();
        let ids: Vec<_> = features.iter().map(|f| f.id.as_str()).collect();
        assert!(ids.contains(&"tls"));
        assert!(ids.contains(&"schema"));
    }

    #[test]
    fn feature_description_returns_some_for_tls() {
        let provider = PostgresqlProvider::new();
        let desc = provider.feature_description("tls");
        assert!(desc.is_some());
        assert!(desc.unwrap().contains("TLS"));
    }

    #[test]
    fn default_signal_is_sigterm() {
        let provider = PostgresqlProvider::new();
        assert_eq!(provider.default_signal(), SIGTERM);
    }

    #[test]
    fn default_args_non_empty_and_definition_includes_flattened_args() {
        let provider = PostgresqlProvider::new();
        let args = provider.default_args();
        assert!(!args.is_empty());
        assert!(args.iter().all(|a| a.name == "-c"));
        let def = provider.definition();
        assert_eq!(def.args.len(), args.len() * 2);
        assert_eq!(def.args.first(), Some(&"-c".to_string()));
        assert_eq!(def.args.get(1), Some(&"shared_buffers=32MB".to_string()));
    }

    #[test]
    fn prepare_for_snapshot_returns_checkpoint_command_over_tcp() {
        let provider = PostgresqlProvider::new();
        let params = ConnectionParams {
            host: "localhost".to_string(),
            port: 5432,
            env: vec![],
        };
        let commands = provider.prepare_for_snapshot(&params).unwrap();
        assert_eq!(commands.len(), 1);
        let cmd = &commands[0];
        assert!(cmd.contains("PGPASSWORD="), "uses password from env");
        assert!(
            cmd.contains("-h 127.0.0.1"),
            "uses TCP to avoid peer auth in docker exec"
        );
        assert!(cmd.contains("$POSTGRES_USER"));
        assert!(cmd.contains("$POSTGRES_DB"));
        assert!(cmd.contains("CHECKPOINT;"));
    }
}
