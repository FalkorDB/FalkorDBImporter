use crate::connector::{ConnectorError, ConnectorResult, DataSourceConfig, DataSourceConnector};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Factory function type for creating connectors
pub type ConnectorFactory =
    Arc<dyn Fn(DataSourceConfig) -> ConnectorResult<Box<dyn DataSourceConnector>> + Send + Sync>;

/// Registry for managing connector implementations
pub struct ConnectorRegistry {
    /// Registered connector factories
    factories: Arc<RwLock<HashMap<String, ConnectorFactory>>>,
}

impl ConnectorRegistry {
    /// Create a new connector registry
    pub fn new() -> Self {
        Self {
            factories: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a connector factory
    pub async fn register<F>(&self, connector_type: String, factory: F) -> ConnectorResult<()>
    where
        F: Fn(DataSourceConfig) -> ConnectorResult<Box<dyn DataSourceConnector>>
            + Send
            + Sync
            + 'static,
    {
        let mut factories = self.factories.write().await;
        factories.insert(connector_type, Arc::new(factory));
        Ok(())
    }

    /// Create a connector instance from configuration
    pub async fn create_connector(
        &self,
        config: DataSourceConfig,
    ) -> ConnectorResult<Box<dyn DataSourceConnector>> {
        let connector_type = config.connection_type().to_string();
        let factories = self.factories.read().await;

        let factory = factories
            .get(&connector_type)
            .ok_or_else(|| {
                ConnectorError::Registry(format!(
                    "No connector registered for type: {}",
                    connector_type
                ))
            })?;

        factory(config)
    }

    /// Check if a connector type is registered
    pub async fn is_registered(&self, connector_type: &str) -> bool {
        let factories = self.factories.read().await;
        factories.contains_key(connector_type)
    }

    /// Get list of registered connector types
    pub async fn registered_types(&self) -> Vec<String> {
        let factories = self.factories.read().await;
        factories.keys().cloned().collect()
    }

    /// Unregister a connector type
    pub async fn unregister(&self, connector_type: &str) -> ConnectorResult<()> {
        let mut factories = self.factories.write().await;
        factories.remove(connector_type);
        Ok(())
    }

    /// Clear all registered connectors
    pub async fn clear(&self) {
        let mut factories = self.factories.write().await;
        factories.clear();
    }
}

impl Default for ConnectorRegistry {
    fn default() -> Self {
        Self::new()
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
        connector_type: String,
    }

    #[async_trait]
    impl DataSourceConnector for MockConnector {
        async fn connect(&mut self) -> ConnectorResult<()> {
            Ok(())
        }

        async fn test_connection(&self) -> ConnectorResult<bool> {
            Ok(true)
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
        ) -> ConnectorResult<BoxStream<'static, ConnectorResult<crate::connector::DataRow>>> {
            Err(ConnectorError::UnsupportedOperation("Mock".to_string()))
        }

        fn connector_type(&self) -> &str {
            &self.connector_type
        }

        async fn disconnect(&mut self) -> ConnectorResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_registry_register_and_create() {
        let registry = ConnectorRegistry::new();

        // Register a mock connector
        registry
            .register("test".to_string(), |_config| {
                Ok(Box::new(MockConnector {
                    connector_type: "test".to_string(),
                }))
            })
            .await
            .unwrap();

        // Check if registered
        assert!(registry.is_registered("test").await);

        // Create a connector
        let config = DataSourceConfig::Generic {
            connector_type: "test".to_string(),
            properties: HashMap::new(),
        };
        let connector = registry.create_connector(config).await.unwrap();
        assert_eq!(connector.connector_type(), "test");
    }

    #[tokio::test]
    async fn test_registry_unregistered_type() {
        let registry = ConnectorRegistry::new();

        let config = DataSourceConfig::Generic {
            connector_type: "nonexistent".to_string(),
            properties: HashMap::new(),
        };

        let result = registry.create_connector(config).await;
        assert!(result.is_err());
        match result {
            Err(ConnectorError::Registry(_)) => {}
            _ => panic!("Expected Registry error"),
        }
    }

    #[tokio::test]
    async fn test_registry_registered_types() {
        let registry = ConnectorRegistry::new();

        registry
            .register("type1".to_string(), |_config| {
                Ok(Box::new(MockConnector {
                    connector_type: "type1".to_string(),
                }))
            })
            .await
            .unwrap();

        registry
            .register("type2".to_string(), |_config| {
                Ok(Box::new(MockConnector {
                    connector_type: "type2".to_string(),
                }))
            })
            .await
            .unwrap();

        let types = registry.registered_types().await;
        assert_eq!(types.len(), 2);
        assert!(types.contains(&"type1".to_string()));
        assert!(types.contains(&"type2".to_string()));
    }

    #[tokio::test]
    async fn test_registry_unregister() {
        let registry = ConnectorRegistry::new();

        registry
            .register("test".to_string(), |_config| {
                Ok(Box::new(MockConnector {
                    connector_type: "test".to_string(),
                }))
            })
            .await
            .unwrap();

        assert!(registry.is_registered("test").await);

        registry.unregister("test").await.unwrap();
        assert!(!registry.is_registered("test").await);
    }

    #[tokio::test]
    async fn test_registry_clear() {
        let registry = ConnectorRegistry::new();

        registry
            .register("type1".to_string(), |_config| {
                Ok(Box::new(MockConnector {
                    connector_type: "type1".to_string(),
                }))
            })
            .await
            .unwrap();

        registry
            .register("type2".to_string(), |_config| {
                Ok(Box::new(MockConnector {
                    connector_type: "type2".to_string(),
                }))
            })
            .await
            .unwrap();

        registry.clear().await;
        assert_eq!(registry.registered_types().await.len(), 0);
    }
}
