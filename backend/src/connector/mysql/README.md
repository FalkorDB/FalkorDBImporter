# MySQL/MariaDB Connector

This connector provides integration with MySQL and MariaDB databases for the FalkorDB Importer.

## Features

- ✅ **Connection with SSL support** - Secure connections to MySQL/MariaDB servers
- ✅ **Schema discovery** - Automatically discover all tables, views, and their structures
- ✅ **Data type mapping** - Comprehensive mapping between MySQL and FalkorDB data types
- ✅ **Data streaming** - Efficient streaming of large datasets

## Supported MySQL Data Types

The connector maps MySQL data types to internal data types as follows:

| MySQL Type | Internal Type |
|------------|---------------|
| TINYINT(1) | Boolean |
| TINYINT, SMALLINT | SmallInt |
| MEDIUMINT, INT | Integer |
| BIGINT | BigInt |
| FLOAT | Float |
| DOUBLE | Double |
| DECIMAL, NUMERIC | Decimal |
| CHAR | Char |
| VARCHAR | Varchar |
| TEXT, TINYTEXT, MEDIUMTEXT, LONGTEXT | Text |
| BLOB, BINARY, VARBINARY | Blob |
| DATETIME | DateTime |
| TIMESTAMP | Timestamp |
| DATE | Date |
| TIME | Time |
| JSON | Json |
| ENUM, SET | String |

## Usage

### Basic Example

```rust
use falkordb_importer_backend::connector::{
    mysql::MysqlConnector,
    DataSourceConfig,
    DataSourceConnector,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a MySQL connection configuration
    let config = DataSourceConfig::Mysql {
        host: "localhost".to_string(),
        port: 3306,
        database: "my_database".to_string(),
        username: "user".to_string(),
        password: "password".to_string(),
        ssl: false, // Set to true to enable SSL
    };

    // Create and connect
    let mut connector = MysqlConnector::new(config)?;
    connector.connect().await?;

    // Discover schema
    let tables = connector.discover_schema().await?;
    println!("Found {} tables", tables.len());

    // Get detailed schema for a specific table
    let table_schema = connector.get_table_schema("users").await?;
    println!("Table has {} columns", table_schema.columns.len());

    // Preview data
    let preview = connector.preview_data("users", 10).await?;
    println!("Preview contains {} rows", preview.len());

    // Stream all data
    use futures::stream::StreamExt;
    let mut stream = connector.stream_data("users").await?;
    while let Some(row_result) = stream.next().await {
        let row = row_result?;
        // Process row...
    }

    // Disconnect
    connector.disconnect().await?;
    Ok(())
}
```

### Using with Connector Registry

```rust
use falkordb_importer_backend::connector::{
    mysql::MysqlConnector,
    ConnectorRegistry,
    DataSourceConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize registry
    let registry = ConnectorRegistry::new();

    // Register MySQL connector
    registry
        .register("mysql".to_string(), |config| {
            Ok(Box::new(MysqlConnector::new(config)?))
        })
        .await?;

    // Create connector from config
    let config = DataSourceConfig::Mysql {
        host: "localhost".to_string(),
        port: 3306,
        database: "my_database".to_string(),
        username: "user".to_string(),
        password: "password".to_string(),
        ssl: false,
    };

    let mut connector = registry.create_connector(config).await?;
    connector.connect().await?;
    
    // Use the connector...
    
    Ok(())
}
```

## Configuration

### Connection Configuration

- **host**: MySQL server hostname or IP address
- **port**: MySQL server port (default: 3306)
- **database**: Name of the database to connect to
- **username**: MySQL username
- **password**: MySQL password
- **ssl**: Enable SSL/TLS encryption (default: false)

### SSL Configuration

When SSL is enabled, the connector uses default SSL options. For production use, you may want to configure:
- SSL certificates
- Certificate verification
- Cipher suites

## Schema Discovery

The connector can discover:
- **Tables** - All base tables in the database
- **Views** - Database views
- **Columns** - Column names, types, nullability, defaults
- **Primary Keys** - Primary key constraints
- **Foreign Keys** - Foreign key relationships
- **Row Count** - Estimated row count from information_schema

## Data Streaming

The `stream_data` method returns a Rust async stream (`BoxStream`) that yields rows one at a time. This is memory-efficient for large datasets.

```rust
use futures::stream::StreamExt;

let mut stream = connector.stream_data("large_table").await?;
let mut count = 0;

while let Some(row_result) = stream.next().await {
    match row_result {
        Ok(row) => {
            count += 1;
            // Process row...
        }
        Err(e) => {
            eprintln!("Stream error: {}", e);
            break;
        }
    }
}

println!("Processed {} rows", count);
```

## Error Handling

The connector uses the `ConnectorError` type for all errors:

```rust
use falkordb_importer_backend::connector::ConnectorError;

match connector.connect().await {
    Ok(_) => println!("Connected!"),
    Err(ConnectorError::Connection(msg)) => {
        eprintln!("Connection failed: {}", msg);
    }
    Err(ConnectorError::Authentication(msg)) => {
        eprintln!("Authentication failed: {}", msg);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

## Testing

Run the tests with:

```bash
cargo test --lib connector::mysql
```

## Running the Example

To run the example (requires a running MySQL/MariaDB instance):

```bash
cargo run --example mysql_connector
```

Note: Update the connection details in the example before running.

## Dependencies

- `mysql_async` - Asynchronous MySQL client
- `async-trait` - Async trait definitions
- `futures` - Stream utilities

## License

Apache-2.0
