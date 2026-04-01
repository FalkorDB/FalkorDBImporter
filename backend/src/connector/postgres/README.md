# PostgreSQL Connector

This connector enables FalkorDB Importer to connect to PostgreSQL databases and import data with full schema discovery.

## Features

- ✅ **SSL/TLS Support**: Secure connections with configurable SSL modes
- ✅ **Schema Discovery**: Automatically discovers tables, views, and materialized views
- ✅ **Column Metadata**: Full column information including types, nullability, and defaults
- ✅ **Key Detection**: Identifies primary keys and foreign key relationships
- ✅ **Data Streaming**: Efficient data retrieval for large tables
- ✅ **Type Mapping**: Comprehensive mapping of PostgreSQL types to internal types

## Usage

### Configuration

Create a PostgreSQL data source configuration:

```rust
use falkordb_importer_backend::connector::{DataSourceConfig, PostgresConnector};

let config = DataSourceConfig::Postgres {
    host: "localhost".to_string(),
    port: 5432,
    database: "mydb".to_string(),
    username: "user".to_string(),
    password: "password".to_string(),
    ssl: true,  // Enable SSL/TLS
    schema: Some("public".to_string()),  // Optional, defaults to "public"
};

let mut connector = PostgresConnector::new(config)?;
```

### Connecting

```rust
// Establish connection
connector.connect().await?;

// Test the connection
let is_connected = connector.test_connection().await?;
```

### Schema Discovery

```rust
// Discover all tables in the schema
let tables = connector.discover_schema().await?;

for table in tables {
    println!("Table: {}", table.name);
    println!("Type: {:?}", table.table_type);
    println!("Columns: {}", table.columns.len());
    println!("Primary Keys: {:?}", table.primary_keys);
    println!("Foreign Keys: {}", table.foreign_keys.len());
}

// Get detailed schema for a specific table
let table_info = connector.get_table_schema("users").await?;
```

### Data Access

```rust
// Preview first 100 rows
let preview = connector.preview_data("users", 100).await?;

// Stream all data from a table
let stream = connector.stream_data("users").await?;
use futures::StreamExt;
while let Some(result) = stream.next().await {
    let row = result?;
    // Process row
}
```

### Registration

The connector is automatically registered when you call:

```rust
use falkordb_importer_backend::connector::{ConnectorRegistry, register_connectors};

let registry = ConnectorRegistry::new();
register_connectors(&registry).await?;

// Now you can create connectors through the registry
let connector = registry.create_connector(config).await?;
```

## Type Mapping

| PostgreSQL Type | Internal Type |
|----------------|---------------|
| `int2`, `smallint`, `smallserial` | `SmallInt` |
| `int4`, `integer`, `serial` | `Integer` |
| `int8`, `bigint`, `bigserial` | `BigInt` |
| `float4`, `real` | `Float` |
| `float8`, `double precision` | `Double` |
| `numeric`, `decimal` | `Decimal` |
| `varchar`, `character varying` | `Varchar` |
| `char`, `character` | `Char` |
| `text` | `Text` |
| `bytea` | `Binary` |
| `date` | `Date` |
| `time` | `Time` |
| `timestamp` | `Timestamp` |
| `timestamptz` | `TimestampTz` |
| `bool`, `boolean` | `Boolean` |
| `json` | `Json` |
| `jsonb` | `JsonB` |
| `uuid` | `Uuid` |
| `xml` | `Xml` |
| `inet` | `Inet` |
| `cidr` | `Cidr` |
| Array types (e.g., `_int4`) | `Array(DataType)` |

## SSL Configuration

The connector supports two SSL modes:

- **`ssl: true`**: Uses `sslmode=require` - connection must use SSL/TLS
- **`ssl: false`**: Uses `sslmode=prefer` - attempts SSL/TLS but falls back to unencrypted

For production environments, always use `ssl: true`.

## Security Considerations

1. **Credentials**: Never log connection strings as they contain passwords
2. **SQL Injection**: The connector properly quotes identifiers to prevent SQL injection
3. **SSL/TLS**: Always use SSL in production to encrypt data in transit
4. **Permissions**: Use database users with minimal required permissions

## Error Handling

The connector provides detailed error messages through the `ConnectorError` enum:

- `Connection`: Connection establishment failures
- `Authentication`: Authentication failures
- `SchemaDiscovery`: Issues discovering schema metadata
- `DataStreaming`: Problems retrieving data
- `TableNotFound`: Requested table doesn't exist

## Testing

Run the PostgreSQL connector tests:

```bash
cd backend
cargo test postgres
```

The test suite includes:
- Configuration validation
- Connection string building
- Type mapping verification
- SSL configuration handling
- Schema naming (public vs custom)

## Requirements

- PostgreSQL 9.5 or later
- Network access to PostgreSQL server
- Valid database credentials with SELECT privileges

## Dependencies

The connector uses:
- `sqlx` with PostgreSQL driver and TLS support
- `tokio` for async runtime
- `futures` for stream handling
