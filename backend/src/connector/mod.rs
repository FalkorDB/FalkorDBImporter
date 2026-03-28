pub mod config;
pub mod error;
pub mod impls;
pub mod pool;
pub mod registry;
pub mod schema;
pub mod types;

use async_trait::async_trait;
use futures::stream::BoxStream;

pub use config::DataSourceConfig;
pub use error::{ConnectorError, ConnectorResult};
#[allow(unused_imports)]
pub use pool::ConnectionPool;
#[allow(unused_imports)]
pub use registry::ConnectorRegistry;
pub use schema::TableInfo;
pub use types::DataRow;

/// Core trait that all data source connectors must implement
#[async_trait]
#[allow(dead_code)]
pub trait DataSourceConnector: Send + Sync {
    /// Establish a connection to the data source
    async fn connect(&mut self) -> ConnectorResult<()>;

    /// Validate credentials and test connection
    async fn test_connection(&self) -> ConnectorResult<bool>;

    /// Discover and list all tables/collections in the data source
    async fn discover_schema(&self) -> ConnectorResult<Vec<TableInfo>>;

    /// Get detailed schema information for a specific table
    async fn get_table_schema(&self, table_name: &str) -> ConnectorResult<TableInfo>;

    /// Fetch a preview of sample rows from a table
    async fn preview_data(&self, table_name: &str, limit: usize) -> ConnectorResult<Vec<DataRow>>;

    /// Stream data from a table asynchronously
    async fn stream_data(
        &self,
        table_name: &str,
    ) -> ConnectorResult<BoxStream<'static, ConnectorResult<DataRow>>>;

    /// Get the connector type name
    fn connector_type(&self) -> &str;

    /// Close the connection and cleanup resources
    async fn disconnect(&mut self) -> ConnectorResult<()>;
}

/// Factory function to create a connector from configuration
#[allow(dead_code)]
pub fn create_connector(config: DataSourceConfig) -> ConnectorResult<Box<dyn DataSourceConnector>> {
    match config {
        #[cfg(feature = "snowflake")]
        DataSourceConfig::Snowflake {
            account,
            warehouse,
            database,
            schema,
            username,
            password,
            private_key_pem,
            private_key_path,
            role,
        } => {
            let connector = impls::SnowflakeConnector::new(
                account,
                warehouse,
                database,
                schema,
                username,
                password,
                private_key_pem,
                private_key_path,
                role,
            );
            Ok(Box::new(connector))
        }
        _ => Err(ConnectorError::UnsupportedOperation(format!(
            "Connector not implemented for type: {}",
            config.connection_type()
        ))),
    }
}
