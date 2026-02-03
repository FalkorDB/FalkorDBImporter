use falkordb_importer_backend::connector::{
    mysql::MysqlConnector, ConnectorRegistry, DataSourceConfig, DataSourceConnector,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the connector registry
    let registry = ConnectorRegistry::new();

    // Register the MySQL connector
    registry
        .register("mysql".to_string(), |config| {
            Ok(Box::new(MysqlConnector::new(config)?))
        })
        .await?;

    // Example 1: Create a MySQL connection configuration
    let mysql_config = DataSourceConfig::Mysql {
        host: "localhost".to_string(),
        port: 3306,
        database: "test_db".to_string(),
        username: "root".to_string(),
        password: "password".to_string(),
        ssl: false,
    };

    // Create a connector instance
    let mut connector = MysqlConnector::new(mysql_config.clone())?;

    // Connect to the database
    println!("Connecting to MySQL database...");
    connector.connect().await?;
    println!("Connected successfully!");

    // Test the connection
    println!("Testing connection...");
    let is_connected = connector.test_connection().await?;
    println!("Connection test result: {}", is_connected);

    // Discover schema
    println!("\nDiscovering database schema...");
    let tables = connector.discover_schema().await?;
    println!("Found {} tables:", tables.len());
    for table in &tables {
        println!("  - {} ({} columns)", table.name, table.columns.len());
    }

    // Get detailed schema for a specific table (if any exist)
    if let Some(table) = tables.first() {
        println!("\nDetailed schema for table '{}':", table.name);
        let detailed_schema = connector.get_table_schema(&table.name).await?;
        println!("Columns:");
        for col in &detailed_schema.columns {
            println!(
                "  - {} ({:?}, nullable: {})",
                col.name, col.data_type, col.nullable
            );
        }
        if !detailed_schema.primary_keys.is_empty() {
            println!("Primary keys: {:?}", detailed_schema.primary_keys);
        }

        // Preview data from the table
        println!("\nPreviewing data from '{}':", table.name);
        let preview = connector.preview_data(&table.name, 5).await?;
        println!("First {} rows:", preview.len());
        for (idx, row) in preview.iter().enumerate() {
            println!("  Row {}: {} columns", idx + 1, row.values.len());
        }
    }

    // Disconnect
    println!("\nDisconnecting...");
    connector.disconnect().await?;
    println!("Disconnected successfully!");

    // Example 2: Using the registry to create a connector
    println!("\n--- Using Registry ---");
    let registered_connector = registry.create_connector(mysql_config).await?;
    println!("Connector type: {}", registered_connector.connector_type());

    Ok(())
}
