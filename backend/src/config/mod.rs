use crate::error::{AppError, AppResult};
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

/// Application configuration
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub falkordb: FalkorDbConfig,
    pub logging: LoggingConfig,
}

/// Server configuration
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub frontend_dir: String,
}

/// FalkorDB configuration
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct FalkorDbConfig {
    pub host: String,
    pub port: u16,
}

/// Logging configuration
#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
}

impl AppConfig {
    /// Load configuration from files and environment variables
    pub fn load() -> AppResult<Self> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let config = Config::builder()
            // Start with default values
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 3000)?
            .set_default("server.frontend_dir", "../frontend/dist")?
            .set_default("falkordb.host", "localhost")?
            .set_default("falkordb.port", 6379)?
            .set_default("logging.level", "info")?
            // Add optional config file
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Override with environment variables (prefix: APP_)
            // Example: APP_SERVER__PORT=8080
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()
            .map_err(|e| AppError::Config(format!("Failed to load configuration: {}", e)))?;

        config
            .try_deserialize()
            .map_err(|e| AppError::Config(format!("Failed to parse configuration: {}", e)))
    }
}

impl From<ConfigError> for AppError {
    fn from(err: ConfigError) -> Self {
        AppError::Config(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::load().expect("Failed to load config");
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.falkordb.host, "localhost");
        assert_eq!(config.falkordb.port, 6379);
        assert_eq!(config.logging.level, "info");
    }
}
