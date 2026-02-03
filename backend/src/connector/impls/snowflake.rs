use crate::connector::schema::{ColumnInfo, TableInfo, TableType};
use crate::connector::types::{DataRow, DataType, DataValue};
use crate::connector::{ConnectorError, ConnectorResult, DataSourceConnector};
use async_trait::async_trait;
use futures::stream::{self, BoxStream, StreamExt};
use serde_json::Value;
use snowflake_api::{QueryResult, SnowflakeApi};
use std::collections::HashMap;

/// Snowflake data warehouse connector
pub struct SnowflakeConnector {
    /// Snowflake API client
    client: Option<SnowflakeApi>,
    /// Account identifier
    account: String,
    /// Warehouse name
    warehouse: String,
    /// Database name
    database: String,
    /// Schema name
    schema: String,
    /// Username
    username: String,
    /// Password (if using password authentication)
    password: Option<String>,
    /// Private key PEM (if using key-pair authentication)
    private_key_pem: Option<String>,
    /// Private key file path (if using key-pair authentication)
    private_key_path: Option<String>,
    /// Role
    role: Option<String>,
}

impl SnowflakeConnector {
    /// Create a new Snowflake connector
    pub fn new(
        account: String,
        warehouse: String,
        database: String,
        schema: String,
        username: String,
        password: Option<String>,
        private_key_pem: Option<String>,
        private_key_path: Option<String>,
        role: Option<String>,
    ) -> Self {
        Self {
            client: None,
            account,
            warehouse,
            database,
            schema,
            username,
            password,
            private_key_pem,
            private_key_path,
            role,
        }
    }

    /// Get the client, returning an error if not connected
    fn get_client(&self) -> ConnectorResult<&SnowflakeApi> {
        self.client
            .as_ref()
            .ok_or_else(|| ConnectorError::Connection("Not connected to Snowflake".to_string()))
    }

    /// Execute a SQL query and return results
    async fn execute_query(&self, query: &str) -> ConnectorResult<QueryResult> {
        let client = self.get_client()?;
        client
            .exec(query)
            .await
            .map_err(|e| ConnectorError::Internal(format!("Query execution failed: {}", e)))
    }

    /// Parse a JSON row array into a HashMap
    #[allow(dead_code)]
    fn parse_json_row(row_value: &Value) -> HashMap<String, Value> {
        let mut row_map = HashMap::new();
        
        if let Some(_row_array) = row_value.as_array() {
            // If the row is an array, we need to use column names
            // For now, just return empty map as we need schema metadata
            // This will be properly handled by accessing result metadata
            return row_map;
        }
        
        if let Some(obj) = row_value.as_object() {
            for (key, value) in obj {
                row_map.insert(key.clone(), value.clone());
            }
        }
        
        row_map
    }

    /// Map Snowflake data type to our DataType
    fn map_data_type(snowflake_type: &str) -> DataType {
        let type_upper = snowflake_type.to_uppercase();
        match type_upper.as_str() {
            t if t.starts_with("NUMBER") || t.starts_with("NUMERIC") => DataType::Numeric,
            t if t.starts_with("DECIMAL") => DataType::Decimal,
            t if t.starts_with("INT") || t == "INTEGER" || t == "BIGINT" => DataType::BigInt,
            t if t.starts_with("SMALLINT") => DataType::SmallInt,
            t if t.starts_with("FLOAT") => DataType::Float,
            t if t.starts_with("DOUBLE") => DataType::Double,
            t if t.starts_with("VARCHAR") || t.starts_with("STRING") || t.starts_with("TEXT") => {
                DataType::Varchar
            }
            t if t.starts_with("CHAR") => DataType::Char,
            t if t.starts_with("BINARY") => DataType::Binary,
            t if t == "BOOLEAN" => DataType::Boolean,
            t if t == "DATE" => DataType::Date,
            t if t == "TIME" => DataType::Time,
            t if t.starts_with("TIMESTAMP") => {
                if t.ends_with("_TZ") || t.ends_with("TZ") && !t.ends_with("NTZ") {
                    DataType::TimestampTz
                } else {
                    DataType::Timestamp
                }
            }
            t if t.starts_with("DATETIME") => DataType::DateTime,
            t if t.starts_with("VARIANT") || t.starts_with("OBJECT") => DataType::Json,
            t if t.starts_with("ARRAY") => DataType::Array(Box::new(DataType::Unknown)),
            _ => DataType::Other(snowflake_type.to_string()),
        }
    }

    /// Convert a JSON value to a DataValue
    fn json_value_to_data_value(&self, value: &Value) -> DataValue {
        match value {
            Value::Null => DataValue::Null,
            Value::Bool(b) => DataValue::Boolean(*b),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    DataValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    DataValue::Float(f)
                } else {
                    DataValue::String(n.to_string())
                }
            }
            Value::String(s) => DataValue::String(s.clone()),
            Value::Array(arr) => {
                let array_values: Vec<DataValue> = arr
                    .iter()
                    .map(|v| self.json_value_to_data_value(v))
                    .collect();
                DataValue::Array(array_values)
            }
            Value::Object(_) => DataValue::String(value.to_string()),
        }
    }
}

#[async_trait]
impl DataSourceConnector for SnowflakeConnector {
    async fn connect(&mut self) -> ConnectorResult<()> {
        // Choose authentication method and create API client
        let api = if let Some(password) = &self.password {
            // Password authentication
            SnowflakeApi::with_password_auth(
                &self.account,
                Some(&self.warehouse),
                Some(&self.database),
                Some(&self.schema),
                &self.username,
                self.role.as_deref(),
                password,
            )
            .map_err(|e| ConnectorError::Authentication(format!("Password auth failed: {}", e)))?
        } else if let Some(private_key_pem) = &self.private_key_pem {
            // Certificate (key-pair) authentication with PEM string
            SnowflakeApi::with_certificate_auth(
                &self.account,
                Some(&self.warehouse),
                Some(&self.database),
                Some(&self.schema),
                &self.username,
                self.role.as_deref(),
                private_key_pem,
            )
            .map_err(|e| {
                ConnectorError::Authentication(format!("Certificate auth failed: {}", e))
            })?
        } else if let Some(private_key_path) = &self.private_key_path {
            // Certificate (key-pair) authentication with key file
            let key_content = std::fs::read_to_string(private_key_path).map_err(|e| {
                ConnectorError::Configuration(format!("Failed to read private key file: {}", e))
            })?;
            SnowflakeApi::with_certificate_auth(
                &self.account,
                Some(&self.warehouse),
                Some(&self.database),
                Some(&self.schema),
                &self.username,
                self.role.as_deref(),
                &key_content,
            )
            .map_err(|e| {
                ConnectorError::Authentication(format!("Certificate auth failed: {}", e))
            })?
        } else {
            return Err(ConnectorError::Configuration(
                "Either password or private key must be provided".to_string(),
            ));
        };

        self.client = Some(api);
        Ok(())
    }

    async fn test_connection(&self) -> ConnectorResult<bool> {
        // Try to execute a simple query
        let _ = self.execute_query("SELECT 1").await?;
        Ok(true)
    }

    async fn discover_schema(&self) -> ConnectorResult<Vec<TableInfo>> {
        let query = format!(
            "SELECT table_name, table_type FROM {}.INFORMATION_SCHEMA.TABLES WHERE table_schema = '{}'",
            self.database, self.schema
        );

        let result = self.execute_query(&query).await?;
        let mut tables = Vec::new();

        // Parse the result - QueryResult is an enum with Json and Arrow variants
        match result {
            QueryResult::Json(json_result) => {
                if let Some(rows) = json_result.value.as_array() {
                    for row in rows {
                        if let Some(row_array) = row.as_array() {
                            // Snowflake returns rows as arrays by default
                            if row_array.len() >= 2 {
                                if let (Some(table_name), Some(table_type_str)) = (
                                    row_array[0].as_str(),
                                    row_array[1].as_str(),
                                ) {
                                    let table_type = match table_type_str.to_uppercase().as_str() {
                                        "VIEW" => TableType::View,
                                        "MATERIALIZED VIEW" => TableType::MaterializedView,
                                        _ => TableType::Table,
                                    };

                                    // Get detailed schema for each table
                                    let table_info = self.get_table_schema(table_name).await?;
                                    tables.push(TableInfo {
                                        table_type,
                                        ..table_info
                                    });
                                }
                            }
                        }
                    }
                }
            }
            QueryResult::Arrow(_) => {
                return Err(ConnectorError::Internal(
                    "Arrow result format not yet supported for schema discovery".to_string(),
                ));
            }
            QueryResult::Empty => {}
        }

        Ok(tables)
    }

    async fn get_table_schema(&self, table_name: &str) -> ConnectorResult<TableInfo> {
        // Get column information
        let query = format!(
            "SELECT column_name, data_type, is_nullable, column_default, numeric_precision, numeric_scale, character_maximum_length
             FROM {}.INFORMATION_SCHEMA.COLUMNS 
             WHERE table_schema = '{}' AND table_name = '{}'
             ORDER BY ordinal_position",
            self.database, self.schema, table_name
        );

        let result = self.execute_query(&query).await?;
        let mut columns = Vec::new();

        match result {
            QueryResult::Json(json_result) => {
                if let Some(rows) = json_result.value.as_array() {
                    for row in rows {
                        if let Some(row_array) = row.as_array() {
                            if row_array.len() >= 7 {
                                let column_name = row_array[0].as_str().unwrap_or("").to_string();
                                let data_type_str = row_array[1].as_str().unwrap_or("VARCHAR");
                                let nullable = row_array[2]
                                    .as_str()
                                    .map(|s| s.to_uppercase() == "YES")
                                    .unwrap_or(true);
                                let default_value = row_array[3].as_str().map(|s| s.to_string());
                                let precision = row_array[4].as_i64().map(|i| i as u32);
                                let scale = row_array[5].as_i64().map(|i| i as u32);
                                let max_length = row_array[6].as_i64().map(|i| i as u32);

                                columns.push(ColumnInfo {
                                    name: column_name,
                                    data_type: Self::map_data_type(data_type_str),
                                    nullable,
                                    is_primary_key: false, // We'll update this below
                                    default_value,
                                    max_length,
                                    precision,
                                    scale,
                                });
                            }
                        }
                    }
                }
            }
            QueryResult::Arrow(_) => {
                return Err(ConnectorError::Internal(
                    "Arrow result format not yet supported for table schema".to_string(),
                ));
            }
            QueryResult::Empty => {}
        }

        // Get primary key information
        let pk_query = format!(
            "SHOW PRIMARY KEYS IN {}.{}.{}",
            self.database, self.schema, table_name
        );

        let primary_keys = match self.execute_query(&pk_query).await {
            Ok(pk_result) => {
                let mut pks = Vec::new();
                if let QueryResult::Json(json_data) = pk_result {
                    if let Some(rows) = json_data.value.as_array() {
                        for row in rows {
                            if let Some(row_array) = row.as_array() {
                                // SHOW PRIMARY KEYS returns: database_name, schema_name, table_name, column_name, key_sequence, constraint_name
                                if row_array.len() >= 4 {
                                    if let Some(col_name) = row_array[3].as_str() {
                                        pks.push(col_name.to_string());
                                        // Mark columns as primary keys
                                        for col in &mut columns {
                                            if col.name == col_name {
                                                col.is_primary_key = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                pks
            }
            Err(_) => Vec::new(), // Table might not have a primary key
        };

        // Get row count estimate
        let count_query = format!(
            "SELECT COUNT(*) as cnt FROM {}.{}.{}",
            self.database, self.schema, table_name
        );

        let row_count = match self.execute_query(&count_query).await {
            Ok(count_result) => {
                if let QueryResult::Json(json_data) = count_result {
                    if let Some(rows) = json_data.value.as_array() {
                        rows.first()
                            .and_then(|row| row.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|v| v.as_i64())
                            .map(|i| i as u64)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Err(_) => None,
        };

        Ok(TableInfo {
            name: table_name.to_string(),
            schema: Some(self.schema.clone()),
            table_type: TableType::Table,
            columns,
            primary_keys,
            foreign_keys: Vec::new(), // Snowflake foreign keys are informational only
            row_count,
        })
    }

    async fn preview_data(&self, table_name: &str, limit: usize) -> ConnectorResult<Vec<DataRow>> {
        let query = format!(
            "SELECT * FROM {}.{}.{} LIMIT {}",
            self.database, self.schema, table_name, limit
        );

        let result = self.execute_query(&query).await?;
        let mut rows = Vec::new();

        match result {
            QueryResult::Json(json_result) => {
                if let Some(json_rows) = json_result.value.as_array() {
                    // Get column names from the result metadata if available
                    // For now, we'll use indices as keys since Snowflake returns arrays
                    for row in json_rows.iter() {
                        if let Some(row_array) = row.as_array() {
                            let mut values = HashMap::new();
                            for (col_idx, value) in row_array.iter().enumerate() {
                                let key = format!("col_{}", col_idx);
                                values.insert(key, self.json_value_to_data_value(value));
                            }
                            rows.push(DataRow { values });
                        }
                    }
                }
            }
            QueryResult::Arrow(_) => {
                return Err(ConnectorError::Internal(
                    "Arrow result format not yet supported for data preview".to_string(),
                ));
            }
            QueryResult::Empty => {}
        }

        Ok(rows)
    }

    async fn stream_data(
        &self,
        table_name: &str,
    ) -> ConnectorResult<BoxStream<'static, ConnectorResult<DataRow>>> {
        let query = format!(
            "SELECT * FROM {}.{}.{}",
            self.database, self.schema, table_name
        );

        let result = self.execute_query(&query).await?;

        // Convert the result into a stream
        let rows: Vec<DataRow> = match result {
            QueryResult::Json(json_result) => {
                if let Some(json_rows) = json_result.value.as_array() {
                    json_rows
                        .iter()
                        .filter_map(|row| {
                            if let Some(row_array) = row.as_array() {
                                let mut values = HashMap::new();
                                for (col_idx, value) in row_array.iter().enumerate() {
                                    let key = format!("col_{}", col_idx);
                                    values.insert(key, self.json_value_to_data_value(value));
                                }
                                Some(DataRow { values })
                            } else {
                                None
                            }
                        })
                        .collect()
                } else {
                    Vec::new()
                }
            }
            QueryResult::Arrow(_) => {
                return Err(ConnectorError::Internal(
                    "Arrow result format not yet supported for data streaming".to_string(),
                ));
            }
            QueryResult::Empty => Vec::new(),
        };

        let stream = stream::iter(rows.into_iter().map(Ok)).boxed();
        Ok(stream)
    }

    fn connector_type(&self) -> &str {
        "snowflake"
    }

    async fn disconnect(&mut self) -> ConnectorResult<()> {
        self.client = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snowflake_connector_creation() {
        let connector = SnowflakeConnector::new(
            "test_account".to_string(),
            "test_warehouse".to_string(),
            "test_database".to_string(),
            "test_schema".to_string(),
            "test_user".to_string(),
            Some("test_password".to_string()),
            None,
            None,
            None,
        );

        assert_eq!(connector.account, "test_account");
        assert_eq!(connector.warehouse, "test_warehouse");
        assert_eq!(connector.database, "test_database");
        assert_eq!(connector.schema, "test_schema");
        assert_eq!(connector.username, "test_user");
        assert!(connector.password.is_some());
        assert_eq!(connector.connector_type(), "snowflake");
    }

    #[test]
    fn test_map_data_type() {
        assert_eq!(
            SnowflakeConnector::map_data_type("NUMBER"),
            DataType::Numeric
        );
        assert_eq!(
            SnowflakeConnector::map_data_type("VARCHAR"),
            DataType::Varchar
        );
        assert_eq!(
            SnowflakeConnector::map_data_type("BOOLEAN"),
            DataType::Boolean
        );
        assert_eq!(SnowflakeConnector::map_data_type("DATE"), DataType::Date);
        assert_eq!(
            SnowflakeConnector::map_data_type("TIMESTAMP_NTZ"),
            DataType::Timestamp
        );
        assert_eq!(
            SnowflakeConnector::map_data_type("TIMESTAMP_TZ"),
            DataType::TimestampTz
        );
    }

    #[test]
    fn test_json_value_to_data_value() {
        let connector = SnowflakeConnector::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            Some("test".to_string()),
            None,
            None,
            None,
        );

        assert_eq!(
            connector.json_value_to_data_value(&Value::Null),
            DataValue::Null
        );
        assert_eq!(
            connector.json_value_to_data_value(&Value::Bool(true)),
            DataValue::Boolean(true)
        );
        assert_eq!(
            connector.json_value_to_data_value(&serde_json::json!(42)),
            DataValue::Integer(42)
        );
        assert_eq!(
            connector.json_value_to_data_value(&serde_json::json!(95.5)),
            DataValue::Float(95.5)
        );
        assert_eq!(
            connector.json_value_to_data_value(&serde_json::json!("Alice")),
            DataValue::String("Alice".to_string())
        );
    }
}
