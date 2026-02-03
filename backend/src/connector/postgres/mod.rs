use crate::connector::{
    ColumnInfo, ConnectorError, ConnectorResult, DataRow, DataSourceConfig,
    DataSourceConnector, DataType, DataValue, ForeignKeyInfo, TableInfo, TableType,
};
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::{Column, Row, TypeInfo, ValueRef};

/// PostgreSQL database connector
pub struct PostgresConnector {
    pool: Option<PgPool>,
    config: PostgresConfig,
}

/// PostgreSQL configuration
#[derive(Debug, Clone)]
struct PostgresConfig {
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String,
    ssl: bool,
    schema: Option<String>,
}

impl PostgresConnector {
    /// Create a new PostgreSQL connector from configuration
    pub fn new(config: DataSourceConfig) -> ConnectorResult<Self> {
        match config {
            DataSourceConfig::Postgres {
                host,
                port,
                database,
                username,
                password,
                ssl,
                schema,
            } => Ok(Self {
                pool: None,
                config: PostgresConfig {
                    host,
                    port,
                    database,
                    username,
                    password,
                    ssl,
                    schema,
                },
            }),
            _ => Err(ConnectorError::Configuration(
                "Invalid configuration for PostgreSQL connector".to_string(),
            )),
        }
    }

    /// Build connection string from configuration
    fn build_connection_string(&self) -> String {
        let ssl_mode = if self.config.ssl {
            "require"
        } else {
            "prefer"
        };

        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            self.config.username,
            self.config.password,
            self.config.host,
            self.config.port,
            self.config.database,
            ssl_mode
        )
    }

    /// Get the schema to query (defaults to 'public')
    fn get_schema(&self) -> String {
        self.config
            .schema
            .clone()
            .unwrap_or_else(|| "public".to_string())
    }

    /// Convert PostgreSQL type to DataType
    fn map_pg_type(type_name: &str) -> DataType {
        match type_name.to_lowercase().as_str() {
            "int2" | "smallint" | "smallserial" => DataType::SmallInt,
            "int4" | "integer" | "serial" => DataType::Integer,
            "int8" | "bigint" | "bigserial" => DataType::BigInt,
            "float4" | "real" => DataType::Float,
            "float8" | "double precision" => DataType::Double,
            "numeric" | "decimal" => DataType::Decimal,
            "varchar" | "character varying" => DataType::Varchar,
            "char" | "character" => DataType::Char,
            "text" => DataType::Text,
            "bytea" => DataType::Binary,
            "date" => DataType::Date,
            "time" | "time without time zone" => DataType::Time,
            "timestamp" | "timestamp without time zone" => DataType::Timestamp,
            "timestamptz" | "timestamp with time zone" => DataType::TimestampTz,
            "bool" | "boolean" => DataType::Boolean,
            "json" => DataType::Json,
            "jsonb" => DataType::JsonB,
            "uuid" => DataType::Uuid,
            "xml" => DataType::Xml,
            "inet" => DataType::Inet,
            "cidr" => DataType::Cidr,
            t if t.starts_with("_") => {
                // Array types in PostgreSQL start with underscore
                let base_type = &t[1..];
                DataType::Array(Box::new(Self::map_pg_type(base_type)))
            }
            _ => DataType::Other(type_name.to_string()),
        }
    }

    /// Convert PostgreSQL row value to DataValue
    fn row_to_data_value(row: &PgRow, column_name: &str, type_name: &str) -> DataValue {
        // Handle NULL values first
        if row.try_get_raw(column_name).unwrap().is_null() {
            return DataValue::Null;
        }

        match type_name.to_lowercase().as_str() {
            "bool" | "boolean" => row
                .try_get::<bool, _>(column_name)
                .map(DataValue::Boolean)
                .unwrap_or(DataValue::Null),
            "int2" | "smallint" | "smallserial" => row
                .try_get::<i16, _>(column_name)
                .map(|v| DataValue::Integer(v as i64))
                .unwrap_or(DataValue::Null),
            "int4" | "integer" | "serial" => row
                .try_get::<i32, _>(column_name)
                .map(|v| DataValue::Integer(v as i64))
                .unwrap_or(DataValue::Null),
            "int8" | "bigint" | "bigserial" => row
                .try_get::<i64, _>(column_name)
                .map(DataValue::Integer)
                .unwrap_or(DataValue::Null),
            "float4" | "real" => row
                .try_get::<f32, _>(column_name)
                .map(|v| DataValue::Float(v as f64))
                .unwrap_or(DataValue::Null),
            "float8" | "double precision" => row
                .try_get::<f64, _>(column_name)
                .map(DataValue::Float)
                .unwrap_or(DataValue::Null),
            "varchar" | "character varying" | "char" | "character" | "text" | "numeric" | "decimal" | "uuid" => row
                .try_get::<String, _>(column_name)
                .map(DataValue::String)
                .unwrap_or(DataValue::Null),
            "bytea" => row
                .try_get::<Vec<u8>, _>(column_name)
                .map(DataValue::Binary)
                .unwrap_or(DataValue::Null),
            "json" | "jsonb" => row
                .try_get::<serde_json::Value, _>(column_name)
                .map(|v| DataValue::String(v.to_string()))
                .unwrap_or(DataValue::Null),
            _ => {
                // Try as string for any other type
                row.try_get::<String, _>(column_name)
                    .map(DataValue::String)
                    .unwrap_or(DataValue::Null)
            }
        }
    }

    /// Convert a row to DataRow
    fn pg_row_to_data_row(row: &PgRow) -> DataRow {
        let mut values = std::collections::HashMap::new();

        for column in row.columns() {
            let column_name = column.name();
            let type_name = column.type_info().name();
            let value = Self::row_to_data_value(row, column_name, type_name);
            values.insert(column_name.to_string(), value);
        }

        DataRow { values }
    }
}

#[async_trait]
impl DataSourceConnector for PostgresConnector {
    async fn connect(&mut self) -> ConnectorResult<()> {
        let connection_string = self.build_connection_string();

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .await
            .map_err(|e| ConnectorError::Connection(format!("Failed to connect: {}", e)))?;

        self.pool = Some(pool);
        Ok(())
    }

    async fn test_connection(&self) -> ConnectorResult<bool> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| ConnectorError::Connection("Not connected".to_string()))?;

        sqlx::query("SELECT 1")
            .execute(pool)
            .await
            .map_err(|e| ConnectorError::Connection(format!("Connection test failed: {}", e)))?;

        Ok(true)
    }

    async fn discover_schema(&self) -> ConnectorResult<Vec<TableInfo>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| ConnectorError::Connection("Not connected".to_string()))?;

        let schema = self.get_schema();

        // Query to get all tables and views in the schema
        let query = r#"
            SELECT 
                c.relname AS table_name,
                CASE c.relkind
                    WHEN 'r' THEN 'table'
                    WHEN 'v' THEN 'view'
                    WHEN 'm' THEN 'materialized_view'
                    ELSE 'other'
                END AS table_type,
                pg_catalog.obj_description(c.oid, 'pg_class') AS table_comment
            FROM pg_catalog.pg_class c
            JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = $1
              AND c.relkind IN ('r', 'v', 'm')
            ORDER BY c.relname
        "#;

        let rows = sqlx::query(query)
            .bind(&schema)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                ConnectorError::SchemaDiscovery(format!("Failed to discover schema: {}", e))
            })?;

        let mut tables = Vec::new();
        for row in rows {
            let table_name: String = row.get("table_name");
            let table_type_str: String = row.get("table_type");

            let table_type = match table_type_str.as_str() {
                "table" => TableType::Table,
                "view" => TableType::View,
                "materialized_view" => TableType::MaterializedView,
                _ => TableType::Other(table_type_str),
            };

            // Get detailed schema for each table
            let table_info = self.get_table_schema(&table_name).await?;
            tables.push(TableInfo {
                name: table_name,
                schema: Some(schema.clone()),
                table_type,
                columns: table_info.columns,
                primary_keys: table_info.primary_keys,
                foreign_keys: table_info.foreign_keys,
                row_count: table_info.row_count,
            });
        }

        Ok(tables)
    }

    async fn get_table_schema(&self, table_name: &str) -> ConnectorResult<TableInfo> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| ConnectorError::Connection("Not connected".to_string()))?;

        let schema = self.get_schema();

        // Query to get column information
        let column_query = r#"
            SELECT 
                a.attname AS column_name,
                pg_catalog.format_type(a.atttypid, a.atttypmod) AS data_type,
                NOT a.attnotnull AS nullable,
                pg_catalog.pg_get_expr(d.adbin, d.adrelid) AS default_value,
                a.attlen AS max_length,
                CASE 
                    WHEN a.atttypid = ANY ('{int,int8,int2}'::regtype[]) 
                    THEN NULL 
                    ELSE a.atttypmod 
                END AS numeric_precision
            FROM pg_catalog.pg_attribute a
            LEFT JOIN pg_catalog.pg_attrdef d ON (a.attrelid = d.adrelid AND a.attnum = d.adnum)
            WHERE a.attrelid = (
                SELECT c.oid 
                FROM pg_catalog.pg_class c
                JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                WHERE n.nspname = $1 AND c.relname = $2
            )
            AND a.attnum > 0 
            AND NOT a.attisdropped
            ORDER BY a.attnum
        "#;

        let rows = sqlx::query(column_query)
            .bind(&schema)
            .bind(table_name)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                ConnectorError::SchemaDiscovery(format!(
                    "Failed to get table schema for '{}': {}",
                    table_name, e
                ))
            })?;

        let mut columns = Vec::new();
        for row in rows {
            let column_name: String = row.get("column_name");
            let data_type_str: String = row.get("data_type");
            let nullable: bool = row.get("nullable");
            let default_value: Option<String> = row.try_get("default_value").ok();

            columns.push(ColumnInfo {
                name: column_name,
                data_type: Self::map_pg_type(&data_type_str),
                nullable,
                is_primary_key: false, // Will be updated below
                default_value,
                max_length: None,
                precision: None,
                scale: None,
            });
        }

        // Query to get primary keys
        let pk_query = r#"
            SELECT a.attname AS column_name
            FROM pg_catalog.pg_index i
            JOIN pg_catalog.pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey)
            WHERE i.indrelid = (
                SELECT c.oid 
                FROM pg_catalog.pg_class c
                JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                WHERE n.nspname = $1 AND c.relname = $2
            )
            AND i.indisprimary
            ORDER BY a.attnum
        "#;

        let pk_rows = sqlx::query(pk_query)
            .bind(&schema)
            .bind(table_name)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                ConnectorError::SchemaDiscovery(format!("Failed to get primary keys: {}", e))
            })?;

        let mut primary_keys = Vec::new();
        for row in pk_rows {
            let column_name: String = row.get("column_name");
            primary_keys.push(column_name.clone());

            // Update column info to mark as primary key
            if let Some(column) = columns.iter_mut().find(|c| c.name == column_name) {
                column.is_primary_key = true;
            }
        }

        // Query to get foreign keys
        let fk_query = r#"
            SELECT
                con.conname AS constraint_name,
                att.attname AS column_name,
                cl_ref.relname AS referenced_table,
                att_ref.attname AS referenced_column
            FROM pg_catalog.pg_constraint con
            JOIN pg_catalog.pg_class cl ON con.conrelid = cl.oid
            JOIN pg_catalog.pg_namespace nsp ON cl.relnamespace = nsp.oid
            JOIN pg_catalog.pg_attribute att ON att.attrelid = cl.oid AND att.attnum = ANY(con.conkey)
            JOIN pg_catalog.pg_class cl_ref ON con.confrelid = cl_ref.oid
            JOIN pg_catalog.pg_attribute att_ref ON att_ref.attrelid = cl_ref.oid AND att_ref.attnum = ANY(con.confkey)
            WHERE nsp.nspname = $1 
              AND cl.relname = $2
              AND con.contype = 'f'
            ORDER BY con.conname, att.attnum
        "#;

        let fk_rows = sqlx::query(fk_query)
            .bind(&schema)
            .bind(table_name)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                ConnectorError::SchemaDiscovery(format!("Failed to get foreign keys: {}", e))
            })?;

        let mut foreign_keys_map: std::collections::HashMap<
            String,
            (Vec<String>, String, Vec<String>),
        > = std::collections::HashMap::new();

        for row in fk_rows {
            let constraint_name: String = row.get("constraint_name");
            let column_name: String = row.get("column_name");
            let referenced_table: String = row.get("referenced_table");
            let referenced_column: String = row.get("referenced_column");

            foreign_keys_map
                .entry(constraint_name.clone())
                .or_insert((Vec::new(), referenced_table.clone(), Vec::new()));

            if let Some((columns, _, ref_cols)) = foreign_keys_map.get_mut(&constraint_name) {
                columns.push(column_name);
                ref_cols.push(referenced_column);
            }
        }

        let foreign_keys = foreign_keys_map
            .into_iter()
            .map(|(name, (columns, referenced_table, referenced_columns))| ForeignKeyInfo {
                name,
                columns,
                referenced_table,
                referenced_columns,
            })
            .collect();

        // Get row count estimate
        let count_query = format!(
            "SELECT reltuples::bigint AS estimate FROM pg_class WHERE oid = '{}.{}'::regclass",
            schema, table_name
        );

        let row_count = sqlx::query(&count_query)
            .fetch_one(pool)
            .await
            .ok()
            .and_then(|row| row.try_get::<i64, _>("estimate").ok())
            .and_then(|v| if v >= 0 { Some(v as u64) } else { None });

        Ok(TableInfo {
            name: table_name.to_string(),
            schema: Some(schema),
            table_type: TableType::Table,
            columns,
            primary_keys,
            foreign_keys,
            row_count,
        })
    }

    async fn preview_data(&self, table_name: &str, limit: usize) -> ConnectorResult<Vec<DataRow>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| ConnectorError::Connection("Not connected".to_string()))?;

        let schema = self.get_schema();
        
        // Simple identifier quoting helper
        let quote_ident = |ident: &str| format!("\"{}\"", ident.replace("\"", "\"\""));
        
        let query = format!(
            "SELECT * FROM {}.{} LIMIT $1",
            quote_ident(&schema),
            quote_ident(table_name)
        );

        let rows = sqlx::query(&query)
            .bind(limit as i64)
            .fetch_all(pool)
            .await
            .map_err(|e| ConnectorError::DataStreaming(format!("Failed to preview data: {}", e)))?;

        Ok(rows.iter().map(Self::pg_row_to_data_row).collect())
    }

    async fn stream_data(
        &self,
        table_name: &str,
    ) -> ConnectorResult<BoxStream<'static, ConnectorResult<DataRow>>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| ConnectorError::Connection("Not connected".to_string()))?
            .clone();

        let schema = self.get_schema();
        
        // Simple identifier quoting helper
        let quote_ident = |ident: &str| format!("\"{}\"", ident.replace("\"", "\"\""));
        
        let query = format!(
            "SELECT * FROM {}.{}",
            quote_ident(&schema),
            quote_ident(table_name)
        );

        // Create a static string by leaking memory for the query
        // This is acceptable for the streaming use case
        let static_query: &'static str = Box::leak(query.into_boxed_str());
        
        // Use cursor-based streaming with fetch
        let stream = sqlx::query(static_query)
            .fetch(&pool)
            .map(|result| {
                result
                    .map(|row| Self::pg_row_to_data_row(&row))
                    .map_err(|e| ConnectorError::DataStreaming(format!("Stream error: {}", e)))
            })
            .boxed();

        Ok(stream)
    }

    fn connector_type(&self) -> &str {
        "postgres"
    }

    async fn disconnect(&mut self) -> ConnectorResult<()> {
        if let Some(pool) = self.pool.take() {
            pool.close().await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_connector_new() {
        let config = DataSourceConfig::Postgres {
            host: "localhost".to_string(),
            port: 5432,
            database: "testdb".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            ssl: false,
            schema: None,
        };

        let connector = PostgresConnector::new(config);
        assert!(connector.is_ok());
    }

    #[test]
    fn test_postgres_connector_wrong_config() {
        let config = DataSourceConfig::Mysql {
            host: "localhost".to_string(),
            port: 3306,
            database: "testdb".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            ssl: false,
        };

        let connector = PostgresConnector::new(config);
        assert!(connector.is_err());
    }

    #[test]
    fn test_build_connection_string() {
        let config = DataSourceConfig::Postgres {
            host: "localhost".to_string(),
            port: 5432,
            database: "testdb".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            ssl: true,
            schema: None,
        };

        let connector = PostgresConnector::new(config).unwrap();
        let conn_str = connector.build_connection_string();
        assert!(conn_str.contains("postgres://user:pass@localhost:5432/testdb"));
        assert!(conn_str.contains("sslmode=require"));
    }

    #[test]
    fn test_build_connection_string_no_ssl() {
        let config = DataSourceConfig::Postgres {
            host: "localhost".to_string(),
            port: 5432,
            database: "testdb".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            ssl: false,
            schema: None,
        };

        let connector = PostgresConnector::new(config).unwrap();
        let conn_str = connector.build_connection_string();
        assert!(conn_str.contains("sslmode=prefer"));
    }

    #[test]
    fn test_map_pg_type() {
        assert_eq!(PostgresConnector::map_pg_type("int4"), DataType::Integer);
        assert_eq!(PostgresConnector::map_pg_type("int8"), DataType::BigInt);
        assert_eq!(PostgresConnector::map_pg_type("text"), DataType::Text);
        assert_eq!(
            PostgresConnector::map_pg_type("varchar"),
            DataType::Varchar
        );
        assert_eq!(PostgresConnector::map_pg_type("boolean"), DataType::Boolean);
        assert_eq!(PostgresConnector::map_pg_type("uuid"), DataType::Uuid);
        assert_eq!(PostgresConnector::map_pg_type("json"), DataType::Json);
        assert_eq!(PostgresConnector::map_pg_type("jsonb"), DataType::JsonB);
        assert_eq!(
            PostgresConnector::map_pg_type("timestamp"),
            DataType::Timestamp
        );
        assert_eq!(
            PostgresConnector::map_pg_type("timestamptz"),
            DataType::TimestampTz
        );

        // Test array types
        match PostgresConnector::map_pg_type("_int4") {
            DataType::Array(inner) => assert_eq!(*inner, DataType::Integer),
            _ => panic!("Expected Array type"),
        }
    }

    #[test]
    fn test_get_schema() {
        let config = DataSourceConfig::Postgres {
            host: "localhost".to_string(),
            port: 5432,
            database: "testdb".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            ssl: false,
            schema: Some("custom_schema".to_string()),
        };

        let connector = PostgresConnector::new(config).unwrap();
        assert_eq!(connector.get_schema(), "custom_schema");
    }

    #[test]
    fn test_get_schema_default() {
        let config = DataSourceConfig::Postgres {
            host: "localhost".to_string(),
            port: 5432,
            database: "testdb".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            ssl: false,
            schema: None,
        };

        let connector = PostgresConnector::new(config).unwrap();
        assert_eq!(connector.get_schema(), "public");
    }
}
