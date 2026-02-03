use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for different types of data sources
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
pub enum DataSourceConfig {
    /// PostgreSQL database configuration
    Postgres {
        host: String,
        port: u16,
        database: String,
        username: String,
        password: String,
        #[serde(default)]
        ssl: bool,
        #[serde(default)]
        schema: Option<String>,
    },

    /// MySQL/MariaDB database configuration
    Mysql {
        host: String,
        port: u16,
        database: String,
        username: String,
        password: String,
        #[serde(default)]
        ssl: bool,
    },

    /// Microsoft SQL Server configuration
    SqlServer {
        host: String,
        port: u16,
        database: String,
        username: String,
        password: String,
        #[serde(default)]
        trusted_connection: bool,
        #[serde(default)]
        encrypt: bool,
    },

    /// Oracle database configuration
    Oracle {
        host: String,
        port: u16,
        service_name: String,
        username: String,
        password: String,
    },

    /// SQLite database configuration
    Sqlite {
        path: String,
        #[serde(default)]
        read_only: bool,
    },

    /// Snowflake data warehouse configuration
    Snowflake {
        account: String,
        warehouse: String,
        database: String,
        schema: String,
        username: String,
        password: String,
        #[serde(default)]
        role: Option<String>,
    },

    /// Databricks configuration
    Databricks {
        host: String,
        http_path: String,
        token: String,
        catalog: String,
        schema: String,
    },

    /// Google BigQuery configuration
    BigQuery {
        project_id: String,
        dataset_id: String,
        credentials_json: String,
    },

    /// Amazon Redshift configuration
    Redshift {
        host: String,
        port: u16,
        database: String,
        username: String,
        password: String,
        #[serde(default)]
        ssl: bool,
    },

    /// AWS S3 configuration
    S3 {
        region: String,
        bucket: String,
        access_key_id: String,
        secret_access_key: String,
        #[serde(default)]
        prefix: Option<String>,
    },

    /// Azure Blob Storage configuration
    AzureBlob {
        account_name: String,
        account_key: String,
        container: String,
        #[serde(default)]
        prefix: Option<String>,
    },

    /// Azure Data Lake configuration
    AzureDataLake {
        account_name: String,
        account_key: String,
        filesystem: String,
        #[serde(default)]
        prefix: Option<String>,
    },

    /// Google Cloud Storage configuration
    GoogleCloudStorage {
        project_id: String,
        bucket: String,
        credentials_json: String,
        #[serde(default)]
        prefix: Option<String>,
    },

    /// Local file configuration
    LocalFile {
        path: String,
        #[serde(default)]
        format: FileFormat,
    },

    /// Generic connector with key-value configuration
    Generic {
        connector_type: String,
        #[serde(flatten)]
        properties: HashMap<String, String>,
    },
}

/// File format for local files
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum FileFormat {
    #[default]
    Csv,
    Tsv,
    Json,
    JsonLines,
    Parquet,
    Excel,
}

impl DataSourceConfig {
    /// Get the connection type identifier
    pub fn connection_type(&self) -> &str {
        match self {
            Self::Postgres { .. } => "postgres",
            Self::Mysql { .. } => "mysql",
            Self::SqlServer { .. } => "sqlserver",
            Self::Oracle { .. } => "oracle",
            Self::Sqlite { .. } => "sqlite",
            Self::Snowflake { .. } => "snowflake",
            Self::Databricks { .. } => "databricks",
            Self::BigQuery { .. } => "bigquery",
            Self::Redshift { .. } => "redshift",
            Self::S3 { .. } => "s3",
            Self::AzureBlob { .. } => "azure_blob",
            Self::AzureDataLake { .. } => "azure_data_lake",
            Self::GoogleCloudStorage { .. } => "google_cloud_storage",
            Self::LocalFile { .. } => "local_file",
            Self::Generic { connector_type, .. } => connector_type,
        }
    }

    /// Validate the configuration
    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Postgres {
                host,
                port,
                database,
                username,
                ..
            } => {
                if host.is_empty() {
                    return Err("Host cannot be empty".to_string());
                }
                if *port == 0 {
                    return Err("Port must be greater than 0".to_string());
                }
                if database.is_empty() {
                    return Err("Database cannot be empty".to_string());
                }
                if username.is_empty() {
                    return Err("Username cannot be empty".to_string());
                }
            }
            Self::Mysql {
                host,
                port,
                database,
                username,
                ..
            } => {
                if host.is_empty() {
                    return Err("Host cannot be empty".to_string());
                }
                if *port == 0 {
                    return Err("Port must be greater than 0".to_string());
                }
                if database.is_empty() {
                    return Err("Database cannot be empty".to_string());
                }
                if username.is_empty() {
                    return Err("Username cannot be empty".to_string());
                }
            }
            Self::Sqlite { path, .. } => {
                if path.is_empty() {
                    return Err("Path cannot be empty".to_string());
                }
            }
            Self::LocalFile { path, .. } => {
                if path.is_empty() {
                    return Err("Path cannot be empty".to_string());
                }
            }
            // Add validation for other types as needed
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_config_type() {
        let config = DataSourceConfig::Postgres {
            host: "localhost".to_string(),
            port: 5432,
            database: "testdb".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            ssl: false,
            schema: None,
        };
        assert_eq!(config.connection_type(), "postgres");
    }

    #[test]
    fn test_postgres_config_validation() {
        let config = DataSourceConfig::Postgres {
            host: "localhost".to_string(),
            port: 5432,
            database: "testdb".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            ssl: false,
            schema: None,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_postgres_config_validation_empty_host() {
        let config = DataSourceConfig::Postgres {
            host: "".to_string(),
            port: 5432,
            database: "testdb".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            ssl: false,
            schema: None,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_mysql_config_type() {
        let config = DataSourceConfig::Mysql {
            host: "localhost".to_string(),
            port: 3306,
            database: "testdb".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            ssl: false,
        };
        assert_eq!(config.connection_type(), "mysql");
    }

    #[test]
    fn test_sqlite_config_validation() {
        let config = DataSourceConfig::Sqlite {
            path: "/tmp/test.db".to_string(),
            read_only: false,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_sqlite_config_validation_empty_path() {
        let config = DataSourceConfig::Sqlite {
            path: "".to_string(),
            read_only: false,
        };
        assert!(config.validate().is_err());
    }
}
