use async_trait::async_trait;
use futures::stream::BoxStream;
use mysql_async::{prelude::*, Conn, OptsBuilder, Pool, Row, SslOpts};
use std::collections::HashMap;

use crate::connector::{
    error::{ConnectorError, ConnectorResult},
    schema::{ColumnInfo, ForeignKeyInfo, TableInfo, TableType},
    types::{DataRow, DataType, DataValue},
    DataSourceConfig, DataSourceConnector,
};

/// MySQL/MariaDB connector implementation
#[allow(dead_code)]
pub struct MysqlConnector {
    pool: Option<Pool>,
    config: MysqlConfig,
}

/// MySQL-specific configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct MysqlConfig {
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String,
    ssl: bool,
}

impl MysqlConnector {
    /// Create a new MySQL connector from configuration
    #[allow(dead_code)]
    pub fn new(config: DataSourceConfig) -> ConnectorResult<Self> {
        match config {
            DataSourceConfig::Mysql {
                host,
                port,
                database,
                username,
                password,
                ssl,
            } => Ok(Self {
                pool: None,
                config: MysqlConfig {
                    host,
                    port,
                    database,
                    username,
                    password,
                    ssl,
                },
            }),
            _ => Err(ConnectorError::Configuration(
                "Invalid configuration type for MySQL connector".to_string(),
            )),
        }
    }

    /// Build MySQL connection options
    #[allow(dead_code)]
    fn build_opts(&self) -> OptsBuilder {
        let mut opts = OptsBuilder::default()
            .ip_or_hostname(&self.config.host)
            .tcp_port(self.config.port)
            .db_name(Some(&self.config.database))
            .user(Some(&self.config.username))
            .pass(Some(&self.config.password));

        // Add SSL configuration if enabled
        if self.config.ssl {
            opts = opts.ssl_opts(Some(SslOpts::default()));
        }

        opts
    }

    /// Get a connection from the pool
    #[allow(dead_code)]
    async fn get_conn(&self) -> ConnectorResult<Conn> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| ConnectorError::Connection("Not connected".to_string()))?;

        pool.get_conn()
            .await
            .map_err(|e| ConnectorError::Connection(format!("Failed to get connection: {}", e)))
    }

    /// Map MySQL column type to DataType
    #[allow(dead_code)]
    fn map_column_type(column_type: &str, _flags: u16) -> DataType {
        let col_type_lower = column_type.to_lowercase();

        match col_type_lower.as_str() {
            // Integer types
            t if t.contains("tinyint(1)") => DataType::Boolean,
            t if t.contains("tinyint") => DataType::SmallInt,
            t if t.contains("smallint") => DataType::SmallInt,
            t if t.contains("mediumint") => DataType::Integer,
            t if t.contains("int") && !t.contains("bigint") => DataType::Integer,
            t if t.contains("bigint") => DataType::BigInt,

            // Floating point types
            t if t.contains("float") => DataType::Float,
            t if t.contains("double") => DataType::Double,
            t if t.contains("decimal") || t.contains("numeric") => DataType::Decimal,

            // String types
            t if t.contains("char") && !t.contains("varchar") => DataType::Char,
            t if t.contains("varchar") => DataType::Varchar,
            t if t.contains("text")
                || t.contains("tinytext")
                || t.contains("mediumtext")
                || t.contains("longtext") =>
            {
                DataType::Text
            }

            // Binary types
            t if t.contains("blob") || t.contains("binary") || t.contains("varbinary") => {
                DataType::Blob
            }

            // Date/Time types
            t if t.contains("datetime") => DataType::DateTime,
            t if t.contains("timestamp") => DataType::Timestamp,
            t if t.contains("date") => DataType::Date,
            t if t.contains("time") => DataType::Time,
            t if t.contains("year") => DataType::SmallInt,

            // JSON type
            t if t.contains("json") => DataType::Json,

            // Enum and Set
            t if t.contains("enum") || t.contains("set") => DataType::String,

            _ => DataType::Other(column_type.to_string()),
        }
    }

    /// Convert MySQL value to DataValue
    #[allow(dead_code)]
    fn convert_value(value: mysql_async::Value) -> DataValue {
        use mysql_async::Value;

        match value {
            Value::NULL => DataValue::Null,
            Value::Bytes(bytes) => {
                // Try to convert to UTF-8 string, otherwise keep as binary
                match String::from_utf8(bytes.clone()) {
                    Ok(s) => DataValue::String(s),
                    Err(_) => DataValue::Binary(bytes),
                }
            }
            Value::Int(i) => DataValue::Integer(i),
            Value::UInt(u) => DataValue::Integer(u as i64),
            Value::Float(f) => DataValue::Float(f as f64),
            Value::Double(d) => DataValue::Float(d),
            Value::Date(year, month, day, hour, minute, second, _micro) => {
                let datetime = format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                    year, month, day, hour, minute, second
                );
                DataValue::String(datetime)
            }
            Value::Time(is_negative, days, hours, minutes, seconds, _micro) => {
                let total_hours = days * 24 + hours as u32;
                let time = if is_negative {
                    format!("-{:02}:{:02}:{:02}", total_hours, minutes, seconds)
                } else {
                    format!("{:02}:{:02}:{:02}", total_hours, minutes, seconds)
                };
                DataValue::String(time)
            }
        }
    }
}

#[async_trait]
impl DataSourceConnector for MysqlConnector {
    async fn connect(&mut self) -> ConnectorResult<()> {
        let opts = self.build_opts();
        let pool = Pool::new(opts);

        // Test the connection
        let mut conn = pool
            .get_conn()
            .await
            .map_err(|e| ConnectorError::Connection(format!("Failed to connect: {}", e)))?;

        // Verify connection by running a simple query
        conn.query_drop("SELECT 1")
            .await
            .map_err(|e| ConnectorError::Connection(format!("Connection test failed: {}", e)))?;

        self.pool = Some(pool);
        Ok(())
    }

    async fn test_connection(&self) -> ConnectorResult<bool> {
        let mut conn = self.get_conn().await?;

        match conn.query_drop("SELECT 1").await {
            Ok(_) => Ok(true),
            Err(e) => Err(ConnectorError::Connection(format!(
                "Connection test failed: {}",
                e
            ))),
        }
    }

    async fn discover_schema(&self) -> ConnectorResult<Vec<TableInfo>> {
        let mut conn = self.get_conn().await?;

        // Query to get all tables in the database
        let query = format!(
            "SELECT TABLE_NAME, TABLE_TYPE, TABLE_ROWS 
             FROM information_schema.TABLES 
             WHERE TABLE_SCHEMA = '{}' 
             ORDER BY TABLE_NAME",
            self.config.database
        );

        let rows: Vec<Row> = conn.query(&query).await.map_err(|e| {
            ConnectorError::SchemaDiscovery(format!("Failed to query tables: {}", e))
        })?;

        let mut tables = Vec::new();
        for row in rows {
            let table_name: String = row.get(0).unwrap();
            let table_type_str: String = row.get(1).unwrap();
            let row_count: Option<u64> = row.get(2);

            let table_type = match table_type_str.as_str() {
                "BASE TABLE" => TableType::Table,
                "VIEW" => TableType::View,
                _ => TableType::Other(table_type_str),
            };

            // Get columns for this table
            let columns = self.get_table_columns(&mut conn, &table_name).await?;
            let primary_keys = self.get_primary_keys(&mut conn, &table_name).await?;
            let foreign_keys = self.get_foreign_keys(&mut conn, &table_name).await?;

            tables.push(TableInfo {
                name: table_name,
                schema: Some(self.config.database.clone()),
                table_type,
                columns,
                primary_keys,
                foreign_keys,
                row_count,
            });
        }

        Ok(tables)
    }

    async fn get_table_schema(&self, table_name: &str) -> ConnectorResult<TableInfo> {
        let mut conn = self.get_conn().await?;

        // Verify table exists and get metadata
        let query = format!(
            "SELECT TABLE_TYPE, TABLE_ROWS 
             FROM information_schema.TABLES 
             WHERE TABLE_SCHEMA = '{}' AND TABLE_NAME = '{}'",
            self.config.database, table_name
        );

        let row: Option<Row> = conn.query_first(&query).await.map_err(|e| {
            ConnectorError::SchemaDiscovery(format!("Failed to query table info: {}", e))
        })?;

        let (table_type, row_count) = match row {
            Some(r) => {
                let table_type_str: String = r.get(0).unwrap();
                let row_count: Option<u64> = r.get(1);
                let table_type = match table_type_str.as_str() {
                    "BASE TABLE" => TableType::Table,
                    "VIEW" => TableType::View,
                    _ => TableType::Other(table_type_str),
                };
                (table_type, row_count)
            }
            None => {
                return Err(ConnectorError::TableNotFound(format!(
                    "Table '{}' not found",
                    table_name
                )))
            }
        };

        let columns = self.get_table_columns(&mut conn, table_name).await?;
        let primary_keys = self.get_primary_keys(&mut conn, table_name).await?;
        let foreign_keys = self.get_foreign_keys(&mut conn, table_name).await?;

        Ok(TableInfo {
            name: table_name.to_string(),
            schema: Some(self.config.database.clone()),
            table_type,
            columns,
            primary_keys,
            foreign_keys,
            row_count,
        })
    }

    async fn preview_data(&self, table_name: &str, limit: usize) -> ConnectorResult<Vec<DataRow>> {
        let mut conn = self.get_conn().await?;

        let query = format!(
            "SELECT * FROM `{}`.`{}` LIMIT {}",
            self.config.database, table_name, limit
        );

        let rows: Vec<Row> = conn
            .query(&query)
            .await
            .map_err(|e| ConnectorError::DataStreaming(format!("Failed to preview data: {}", e)))?;

        let mut result = Vec::new();
        for row in rows {
            let mut values = HashMap::new();
            for (idx, column) in row.columns_ref().iter().enumerate() {
                let col_name = column.name_str().to_string();
                let value = row.as_ref(idx).unwrap();
                values.insert(col_name, Self::convert_value(value.clone()));
            }
            result.push(DataRow { values });
        }

        Ok(result)
    }

    async fn stream_data(
        &self,
        table_name: &str,
    ) -> ConnectorResult<BoxStream<'static, ConnectorResult<DataRow>>> {
        let mut conn = self.get_conn().await?;

        let query = format!("SELECT * FROM `{}`.`{}`", self.config.database, table_name);

        // For now, we'll collect all rows and convert to stream
        // This is a simplified implementation
        let rows: Vec<Row> = conn
            .query(query)
            .await
            .map_err(|e| ConnectorError::DataStreaming(format!("Failed to query data: {}", e)))?;

        // Convert rows to DataRows
        let data_rows: Vec<ConnectorResult<DataRow>> = rows
            .into_iter()
            .map(|row| {
                let mut values = HashMap::new();
                for (idx, column) in row.columns_ref().iter().enumerate() {
                    let col_name = column.name_str().to_string();
                    let value = row.as_ref(idx).unwrap();
                    values.insert(col_name, Self::convert_value(value.clone()));
                }
                Ok(DataRow { values })
            })
            .collect();

        // Convert to stream
        let stream = futures::stream::iter(data_rows);
        Ok(Box::pin(stream))
    }

    fn connector_type(&self) -> &str {
        "mysql"
    }

    async fn disconnect(&mut self) -> ConnectorResult<()> {
        if let Some(pool) = self.pool.take() {
            pool.disconnect()
                .await
                .map_err(|e| ConnectorError::Connection(format!("Failed to disconnect: {}", e)))?;
        }
        Ok(())
    }
}

// Helper methods for MysqlConnector
impl MysqlConnector {
    /// Get column information for a table
    #[allow(dead_code)]
    async fn get_table_columns(
        &self,
        conn: &mut Conn,
        table_name: &str,
    ) -> ConnectorResult<Vec<ColumnInfo>> {
        let query = format!(
            "SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE, COLUMN_KEY, 
                    COLUMN_DEFAULT, CHARACTER_MAXIMUM_LENGTH, 
                    NUMERIC_PRECISION, NUMERIC_SCALE
             FROM information_schema.COLUMNS 
             WHERE TABLE_SCHEMA = '{}' AND TABLE_NAME = '{}' 
             ORDER BY ORDINAL_POSITION",
            self.config.database, table_name
        );

        let rows: Vec<Row> = conn.query(&query).await.map_err(|e| {
            ConnectorError::SchemaDiscovery(format!("Failed to query columns: {}", e))
        })?;

        let mut columns = Vec::new();
        for row in rows {
            let name: String = row.get(0).unwrap();
            let column_type: String = row.get(1).unwrap();
            let is_nullable: String = row.get(2).unwrap();
            let column_key: String = row.get(3).unwrap();
            let default_value: Option<String> = row.get(4);
            let max_length: Option<u32> = row.get(5);
            let precision: Option<u32> = row.get(6);
            let scale: Option<u32> = row.get(7);

            let data_type = Self::map_column_type(&column_type, 0);
            let nullable = is_nullable == "YES";
            let is_primary_key = column_key == "PRI";

            columns.push(ColumnInfo {
                name,
                data_type,
                nullable,
                is_primary_key,
                default_value,
                max_length,
                precision,
                scale,
            });
        }

        Ok(columns)
    }

    /// Get primary key columns for a table
    #[allow(dead_code)]
    async fn get_primary_keys(
        &self,
        conn: &mut Conn,
        table_name: &str,
    ) -> ConnectorResult<Vec<String>> {
        let query = format!(
            "SELECT COLUMN_NAME 
             FROM information_schema.KEY_COLUMN_USAGE 
             WHERE TABLE_SCHEMA = '{}' AND TABLE_NAME = '{}' 
               AND CONSTRAINT_NAME = 'PRIMARY' 
             ORDER BY ORDINAL_POSITION",
            self.config.database, table_name
        );

        let rows: Vec<Row> = conn.query(&query).await.map_err(|e| {
            ConnectorError::SchemaDiscovery(format!("Failed to query primary keys: {}", e))
        })?;

        let primary_keys = rows
            .into_iter()
            .map(|row| row.get::<String, _>(0).unwrap())
            .collect();

        Ok(primary_keys)
    }

    /// Get foreign key relationships for a table
    #[allow(dead_code)]
    async fn get_foreign_keys(
        &self,
        conn: &mut Conn,
        table_name: &str,
    ) -> ConnectorResult<Vec<ForeignKeyInfo>> {
        let query = format!(
            "SELECT CONSTRAINT_NAME, COLUMN_NAME, 
                    REFERENCED_TABLE_NAME, REFERENCED_COLUMN_NAME
             FROM information_schema.KEY_COLUMN_USAGE 
             WHERE TABLE_SCHEMA = '{}' AND TABLE_NAME = '{}' 
               AND REFERENCED_TABLE_NAME IS NOT NULL 
             ORDER BY CONSTRAINT_NAME, ORDINAL_POSITION",
            self.config.database, table_name
        );

        let rows: Vec<Row> = conn.query(&query).await.map_err(|e| {
            ConnectorError::SchemaDiscovery(format!("Failed to query foreign keys: {}", e))
        })?;

        let mut fk_map: HashMap<String, ForeignKeyInfo> = HashMap::new();

        for row in rows {
            let constraint_name: String = row.get(0).unwrap();
            let column_name: String = row.get(1).unwrap();
            let referenced_table: String = row.get(2).unwrap();
            let referenced_column: String = row.get(3).unwrap();

            let entry = fk_map
                .entry(constraint_name.clone())
                .or_insert_with(|| ForeignKeyInfo {
                    name: constraint_name.clone(),
                    columns: Vec::new(),
                    referenced_table,
                    referenced_columns: Vec::new(),
                });

            entry.columns.push(column_name);
            entry.referenced_columns.push(referenced_column);
        }

        Ok(fk_map.into_values().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_column_type() {
        assert_eq!(
            MysqlConnector::map_column_type("int(11)", 0),
            DataType::Integer
        );
        assert_eq!(
            MysqlConnector::map_column_type("bigint(20)", 0),
            DataType::BigInt
        );
        assert_eq!(
            MysqlConnector::map_column_type("varchar(255)", 0),
            DataType::Varchar
        );
        assert_eq!(MysqlConnector::map_column_type("text", 0), DataType::Text);
        assert_eq!(
            MysqlConnector::map_column_type("datetime", 0),
            DataType::DateTime
        );
        assert_eq!(
            MysqlConnector::map_column_type("timestamp", 0),
            DataType::Timestamp
        );
        assert_eq!(
            MysqlConnector::map_column_type("tinyint(1)", 0),
            DataType::Boolean
        );
        assert_eq!(
            MysqlConnector::map_column_type("decimal(10,2)", 0),
            DataType::Decimal
        );
        assert_eq!(MysqlConnector::map_column_type("json", 0), DataType::Json);
    }

    #[test]
    fn test_mysql_connector_creation() {
        let config = DataSourceConfig::Mysql {
            host: "localhost".to_string(),
            port: 3306,
            database: "test".to_string(),
            username: "root".to_string(),
            password: "password".to_string(),
            ssl: false,
        };

        let connector = MysqlConnector::new(config);
        assert!(connector.is_ok());

        let connector = connector.unwrap();
        assert_eq!(connector.connector_type(), "mysql");
    }

    #[test]
    fn test_mysql_connector_invalid_config() {
        let config = DataSourceConfig::Postgres {
            host: "localhost".to_string(),
            port: 5432,
            database: "test".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            ssl: false,
            schema: None,
        };

        let result = MysqlConnector::new(config);
        assert!(result.is_err());
        match result {
            Err(ConnectorError::Configuration(_)) => {}
            _ => panic!("Expected Configuration error"),
        }
    }

    #[test]
    fn test_convert_value_null() {
        use mysql_async::Value;
        let value = Value::NULL;
        let result = MysqlConnector::convert_value(value);
        assert_eq!(result, DataValue::Null);
    }

    #[test]
    fn test_convert_value_int() {
        use mysql_async::Value;
        let value = Value::Int(42);
        let result = MysqlConnector::convert_value(value);
        assert_eq!(result, DataValue::Integer(42));
    }

    #[test]
    fn test_convert_value_string() {
        use mysql_async::Value;
        let value = Value::Bytes(b"hello".to_vec());
        let result = MysqlConnector::convert_value(value);
        assert_eq!(result, DataValue::String("hello".to_string()));
    }
}
