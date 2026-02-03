use crate::connector::{ConnectorError, ConnectorResult, DataSourceConnector};
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

/// Configuration for connection pooling
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PoolConfig {
    /// Maximum number of connections in the pool
    pub max_connections: usize,

    /// Minimum number of idle connections to maintain
    pub min_idle: usize,

    /// Maximum lifetime of a connection in seconds
    pub max_lifetime: Option<u64>,

    /// Timeout for acquiring a connection in seconds
    pub connection_timeout: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_idle: 2,
            max_lifetime: Some(3600), // 1 hour
            connection_timeout: 30,
        }
    }
}

/// Connection pool for managing data source connections
pub struct ConnectionPool<T: DataSourceConnector> {
    /// Pool configuration
    #[allow(dead_code)]
    config: PoolConfig,

    /// Available connections
    connections: Arc<Mutex<Vec<Arc<Mutex<T>>>>>,

    /// Semaphore to limit concurrent connections
    semaphore: Arc<Semaphore>,

    /// Connection factory function
    factory: Arc<dyn Fn() -> Result<T, ConnectorError> + Send + Sync>,
}

impl<T: DataSourceConnector + 'static> ConnectionPool<T> {
    /// Create a new connection pool
    #[allow(dead_code)]
    pub fn new<F>(config: PoolConfig, factory: F) -> Self
    where
        F: Fn() -> Result<T, ConnectorError> + Send + Sync + 'static,
    {
        let semaphore = Arc::new(Semaphore::new(config.max_connections));
        Self {
            config,
            connections: Arc::new(Mutex::new(Vec::new())),
            semaphore,
            factory: Arc::new(factory),
        }
    }

    /// Get a connection from the pool
    #[allow(dead_code)]
    pub async fn acquire(&self) -> ConnectorResult<PooledConnection<T>> {
        // Acquire a permit from the semaphore
        let permit = self.semaphore.clone().acquire_owned().await.map_err(|e| {
            ConnectorError::ConnectionPool(format!("Failed to acquire permit: {}", e))
        })?;

        // Try to get an existing connection
        let mut connections = self.connections.lock().await;
        if let Some(conn) = connections.pop() {
            drop(connections); // Release the lock
            return Ok(PooledConnection {
                connection: Some(conn),
                pool: self.connections.clone(),
                _permit: permit,
            });
        }
        drop(connections); // Release the lock

        // Create a new connection
        let mut connector = (self.factory)().map_err(|e| {
            ConnectorError::ConnectionPool(format!("Failed to create connector: {}", e))
        })?;

        connector.connect().await?;

        Ok(PooledConnection {
            connection: Some(Arc::new(Mutex::new(connector))),
            pool: self.connections.clone(),
            _permit: permit,
        })
    }

    /// Get the current size of the pool
    #[allow(dead_code)]
    pub async fn size(&self) -> usize {
        self.connections.lock().await.len()
    }

    /// Close all connections in the pool
    #[allow(dead_code)]
    pub async fn close(&self) -> ConnectorResult<()> {
        let mut connections = self.connections.lock().await;
        for conn in connections.drain(..) {
            let mut c = conn.lock().await;
            c.disconnect().await?;
        }
        Ok(())
    }
}

/// A pooled connection that returns to the pool when dropped
pub struct PooledConnection<T: DataSourceConnector + 'static> {
    connection: Option<Arc<Mutex<T>>>,
    pool: Arc<Mutex<Vec<Arc<Mutex<T>>>>>,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl<T: DataSourceConnector + 'static> PooledConnection<T> {
    /// Get a reference to the underlying connector
    #[allow(dead_code)]
    pub async fn get(&self) -> Arc<Mutex<T>> {
        self.connection.as_ref().unwrap().clone()
    }
}

impl<T: DataSourceConnector + 'static> Drop for PooledConnection<T> {
    fn drop(&mut self) {
        if let Some(conn) = self.connection.take() {
            let pool = self.pool.clone();
            tokio::spawn(async move {
                let mut connections = pool.lock().await;
                connections.push(conn);
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::TableInfo;
    use async_trait::async_trait;
    use futures::stream::BoxStream;

    // Mock connector for testing
    struct MockConnector {
        connected: bool,
    }

    #[async_trait]
    impl DataSourceConnector for MockConnector {
        async fn connect(&mut self) -> ConnectorResult<()> {
            self.connected = true;
            Ok(())
        }

        async fn test_connection(&self) -> ConnectorResult<bool> {
            Ok(self.connected)
        }

        async fn discover_schema(&self) -> ConnectorResult<Vec<TableInfo>> {
            Ok(vec![])
        }

        async fn get_table_schema(&self, _table_name: &str) -> ConnectorResult<TableInfo> {
            Err(ConnectorError::UnsupportedOperation("Mock".to_string()))
        }

        async fn preview_data(
            &self,
            _table_name: &str,
            _limit: usize,
        ) -> ConnectorResult<Vec<crate::connector::DataRow>> {
            Ok(vec![])
        }

        async fn stream_data(
            &self,
            _table_name: &str,
        ) -> ConnectorResult<BoxStream<'static, ConnectorResult<crate::connector::DataRow>>>
        {
            Err(ConnectorError::UnsupportedOperation("Mock".to_string()))
        }

        fn connector_type(&self) -> &str {
            "mock"
        }

        async fn disconnect(&mut self) -> ConnectorResult<()> {
            self.connected = false;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_pool_acquire_and_return() {
        let config = PoolConfig {
            max_connections: 5,
            min_idle: 1,
            max_lifetime: None,
            connection_timeout: 10,
        };

        let pool = ConnectionPool::new(config, || Ok(MockConnector { connected: false }));

        // Acquire a connection
        let conn = pool.acquire().await.unwrap();
        assert_eq!(pool.size().await, 0); // Connection is in use

        // Return connection to pool
        drop(conn);
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        assert_eq!(pool.size().await, 1); // Connection returned to pool
    }

    #[tokio::test]
    async fn test_pool_multiple_connections() {
        let config = PoolConfig {
            max_connections: 3,
            ..Default::default()
        };

        let pool = ConnectionPool::new(config, || Ok(MockConnector { connected: false }));

        let conn1 = pool.acquire().await.unwrap();
        let conn2 = pool.acquire().await.unwrap();
        let conn3 = pool.acquire().await.unwrap();

        assert_eq!(pool.size().await, 0);

        drop(conn1);
        drop(conn2);
        drop(conn3);

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert_eq!(pool.size().await, 3);
    }
}
