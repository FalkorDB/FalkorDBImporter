pub mod config;
pub mod error;
pub mod pool;
pub mod postgres;
pub mod registry;
pub mod schema;
pub mod types;

use async_trait::async_trait;
use futures::stream::BoxStream;

pub use config::DataSourceConfig;
pub use error::{ConnectorError, ConnectorResult};
#[allow(unused_imports)]
pub use pool::ConnectionPool;
pub use postgres::PostgresConnector;
pub use registry::ConnectorRegistry;
pub use schema::{ColumnInfo, ForeignKeyInfo, TableInfo, TableType};
pub use types::{DataRow, DataType, DataValue};

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

/// Initialize and register all available connectors
pub async fn register_connectors(registry: &ConnectorRegistry) -> ConnectorResult<()> {
    // Register PostgreSQL connector
    registry
        .register("postgres".to_string(), |config| {
            let connector = PostgresConnector::new(config)?;
            Ok(Box::new(connector) as Box<dyn DataSourceConnector>)
        })
        .await?;

    tracing::info!("PostgreSQL connector registered");

    Ok(())
}
