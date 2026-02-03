use thiserror::Error;

/// Errors that can occur when working with data source connectors
#[derive(Error, Debug)]
pub enum ConnectorError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Schema discovery error: {0}")]
    SchemaDiscovery(String),

    #[error("Data streaming error: {0}")]
    DataStreaming(String),

    #[error("Table not found: {0}")]
    TableNotFound(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Connection pool error: {0}")]
    ConnectionPool(String),

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias for connector operations
pub type ConnectorResult<T> = Result<T, ConnectorError>;
