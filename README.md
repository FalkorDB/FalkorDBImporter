# FalkorDBImporter
FalkorDB Importer


# FalkorDB Data Importer
## Project Plan & Task Breakdown

**February 2026**

---

## 1. Executive Summary

This document outlines the project plan for building a Data Importer service for FalkorDB. The service will provide a high-performance, scalable tool for importing data from multiple sources into FalkorDB, with visual data modeling and mapping capabilities.

**Key Design Decisions:**
- **Rust Backend** for maximum performance, memory safety, and scalability
- **Multi-Database Connectivity** supporting enterprise data sources
- **Pluggable Connector Architecture** for extensibility

The FalkorDB Data Importer will enable users to:

- Connect to and import from relational databases (PostgreSQL, MySQL, SQL Server, Oracle)
- Connect to cloud data warehouses (Snowflake, Databricks, BigQuery)
- Import from cloud storage (AWS S3, Azure Blob/Data Lake, Google Cloud Storage)
- Upload local files (CSV, JSON, TSV, Parquet)
- Visually design graph data models with nodes and relationships
- Map source columns/fields to graph model properties
- Configure data type transformations and ID mappings
- Preview and execute imports with progress tracking
- Save and load mapping configurations for reuse

---

## 2. Project Overview

### 2.1 Supported Data Sources

| Category | Sources |
|----------|---------|
| **Relational Databases** | PostgreSQL, MySQL, SQL Server, Oracle, MariaDB, SQLite |
| **Cloud Data Warehouses** | Snowflake, Databricks, Google BigQuery, Amazon Redshift |
| **Cloud Storage** | AWS S3, Azure Blob Storage, Azure Data Lake, Google Cloud Storage |
| **Local Files** | CSV, TSV, JSON, JSON Lines, Parquet, Excel (XLSX) |

### 2.2 Core Features

| Feature | Description |
|---------|-------------|
| **Data Source Connection** | Connect to databases, warehouses, and cloud storage with credential management |
| **Schema Discovery** | Auto-discover tables, columns, and relationships from source databases |
| **File Provision** | Upload local files or browse cloud storage; preview contents |
| **Data Modeling** | Visual canvas for creating nodes and relationships; define labels, types, and properties |
| **Mapping** | Map source tables/files to graph elements; column-to-property mapping with type inference |
| **SQL Query Builder** | Write custom SQL queries for complex data extraction |
| **ID Configuration** | Define unique identifiers; composite keys; ID generation strategies |
| **Type Conversion** | Automatic type detection; manual override for dates, numbers, booleans, etc. |
| **Indexes/Constraints** | Create indexes and constraints during import for query optimization |
| **Import Execution** | Preview Cypher queries; parallel batch import with progress tracking |
| **Configuration Export** | Save/load mapping configurations as JSON for reuse and version control |
| **Scheduling** | Schedule recurring imports (future enhancement) |

### 2.3 Technical Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Frontend (React/TypeScript)                   │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │ Source   │ │ Schema   │ │ Model    │ │ Mapping  │ │ Import   │  │
│  │ Connect  │ │ Browser  │ │ Canvas   │ │ Panel    │ │ Runner   │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                                   │
                            REST/WebSocket API
                                   │
┌─────────────────────────────────────────────────────────────────────┐
│                      Backend API (Rust/Axum)                         │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    Connection Manager                         │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ │   │
│  │  │Postgres │ │ MySQL   │ │Snowflake│ │  S3     │ │  CSV    │ │   │
│  │  │Connector│ │Connector│ │Connector│ │Connector│ │ Parser  │ │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘ │   │
│  └──────────────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Schema      │  Cypher      │  Import       │  Config        │   │
│  │  Discovery   │  Generator   │  Engine       │  Manager       │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                                   │
                              FalkorDB Client
                                   │
                         ┌─────────────────────┐
                         │     FalkorDB        │
                         └─────────────────────┘
```

### 2.4 Why Rust for the Backend?

| Benefit | Description |
|---------|-------------|
| **Performance** | Near-C performance for data processing; critical for large imports |
| **Memory Safety** | No null pointer exceptions, buffer overflows, or data races |
| **Concurrency** | Fearless concurrency with async/await (Tokio); parallel data streaming |
| **Low Memory Footprint** | Efficient memory usage for processing large datasets |
| **Type Safety** | Strong typing catches errors at compile time |
| **Ecosystem** | Excellent database drivers (sqlx, tokio-postgres, mysql_async) |
| **Deployment** | Single binary deployment; minimal runtime dependencies |

---

## 3. Phase 1: Foundation & Core Infrastructure

**Estimated Duration: 4-5 weeks**

### 3.1 Project Setup

1. Initialize monorepo structure (frontend, backend, shared types)
2. Set up React/TypeScript frontend with Vite
3. Set up Rust backend with Axum web framework
4. Configure Cargo workspace with shared crates
5. Set up Docker development environment with FalkorDB
6. Configure CI/CD pipeline (GitHub Actions) with Rust tests
7. Set up code formatting (rustfmt) and linting (clippy)

### 3.2 Rust Backend Core

8. Define async runtime configuration (Tokio)
9. Create API router structure with Axum
10. Implement error handling with thiserror and anyhow
11. Set up logging with tracing crate
12. Create configuration management (config crate)
13. Implement graceful shutdown handling
14. Set up OpenAPI documentation (utoipa)

### 3.3 Connector Trait Architecture

15. Define `DataSourceConnector` trait with async methods:
    - `connect()` - Establish connection
    - `test_connection()` - Validate credentials
    - `discover_schema()` - List tables/collections
    - `get_table_schema()` - Get columns and types
    - `preview_data()` - Fetch sample rows
    - `stream_data()` - Async data streaming
16. Define `DataSourceConfig` enum for connection parameters
17. Create connector registry for dynamic connector loading
18. Implement connection pooling abstraction

### 3.4 FalkorDB Integration

19. Integrate FalkorDB Rust client (or create bindings)
20. Implement connection configuration and testing
21. Create Cypher query executor with parameterized queries
22. Implement batch transaction support
23. Add connection pooling for FalkorDB

---

## 4. Phase 2: Data Source Connectors

**Estimated Duration: 5-6 weeks**

### 4.1 Relational Database Connectors

24. **PostgreSQL Connector** (using sqlx or tokio-postgres)
    - Connection with SSL support
    - Schema discovery (tables, views, columns, types)
    - Primary key and foreign key detection
    - Data streaming with cursor-based pagination
25. **MySQL/MariaDB Connector** (using mysql_async)
    - Connection with SSL support
    - Schema discovery
    - Data type mapping
    - Data streaming
26. **SQL Server Connector** (using tiberius)
    - Windows and SQL authentication
    - Schema discovery
    - Data streaming
27. **Oracle Connector** (using oracle crate or ODBC)
    - Connection configuration
    - Schema discovery
    - Data streaming
28. **SQLite Connector** (using rusqlite)
    - File-based connection
    - Schema discovery

### 4.2 Cloud Data Warehouse Connectors

29. **Snowflake Connector**
    - OAuth and key-pair authentication
    - Warehouse/database/schema selection
    - Schema discovery
    - Query-based data extraction
    - Large result set streaming
30. **Google BigQuery Connector**
    - Service account authentication
    - Dataset/table discovery
    - Query execution
    - Result streaming
31. **Databricks Connector**
    - Token-based authentication
    - Catalog/schema/table discovery
    - SQL warehouse query execution
32. **Amazon Redshift Connector** (via PostgreSQL protocol)
    - IAM and password authentication
    - Schema discovery
    - UNLOAD for large exports (optional)

### 4.3 Cloud Storage Connectors

33. **AWS S3 Connector** (using aws-sdk-s3)
    - IAM and access key authentication
    - Bucket/prefix browsing
    - File listing and metadata
    - Streaming file download
    - Support for CSV, JSON, Parquet files
34. **Azure Blob Storage Connector** (using azure_storage_blobs)
    - SAS token and connection string auth
    - Container/blob browsing
    - File streaming
35. **Azure Data Lake Storage Gen2 Connector**
    - Service principal authentication
    - Hierarchical namespace browsing
36. **Google Cloud Storage Connector** (using cloud-storage)
    - Service account authentication
    - Bucket/object browsing
    - File streaming

### 4.4 File Parsers

37. **CSV Parser** (using csv crate)
    - Header detection
    - Delimiter inference (comma, semicolon, tab, pipe)
    - Quote character handling
    - Streaming large files
    - Type inference per column
38. **JSON Parser** (using serde_json)
    - Object and array detection
    - Nested structure flattening
    - JSON Lines support
    - Streaming parser for large files
39. **Parquet Parser** (using parquet crate)
    - Schema extraction
    - Column selection
    - Row group streaming
40. **Excel Parser** (using calamine)
    - Sheet discovery
    - Header detection
    - Data extraction

---

## 5. Phase 3: Frontend & Data Source UI

**Estimated Duration: 4-5 weeks**

### 5.1 Data Source Connection UI

41. Create data source type selector (database, warehouse, storage, file)
42. Build connection form components for each source type
43. Implement credential input with secure handling
44. Add connection testing with status feedback
45. Create saved connections management (list, edit, delete)
46. Implement connection encryption for stored credentials

### 5.2 Schema Browser

47. Build tree view for database/schema/table navigation
48. Display table columns with data types
49. Show primary key and foreign key indicators
50. Implement table search and filtering
51. Add table preview (sample data)
52. Support custom SQL query input

### 5.3 Cloud Storage Browser

53. Build bucket/container browser
54. Implement folder navigation
55. Display file metadata (size, type, modified date)
56. Add file preview for supported formats
57. Implement file search with prefix filtering

### 5.4 Local File Upload

58. Create drag-and-drop upload zone
59. Implement multi-file upload support
60. Add upload progress tracking
61. Build file preview component with pagination
62. Display inferred schema (columns and types)

---

## 6. Phase 4: Visual Data Modeling

**Estimated Duration: 4-5 weeks**

### 6.1 Graph Canvas Implementation

63. Integrate graph visualization library (React Flow recommended)
64. Implement node creation and editing
65. Implement relationship creation with directional arrows
66. Add drag-and-drop node positioning
67. Implement zoom, pan, and canvas controls
68. Add multi-select and bulk operations (delete, move)
69. Implement undo/redo functionality
70. Add minimap for large models

### 6.2 Node Configuration

71. Create node label editor (single and multiple labels)
72. Build property definition panel
73. Implement ID property selection with validation
74. Add data type selector for each property
75. Support property constraints (required, unique)
76. Add node color/icon customization

### 6.3 Relationship Configuration

77. Create relationship type editor
78. Implement source/target node selection
79. Add relationship property definitions
80. Support self-referencing relationships
81. Handle multiple relationships between same nodes

### 6.4 Schema Auto-Discovery

82. Generate suggested graph model from relational schema
83. Detect potential nodes from tables
84. Detect potential relationships from foreign keys
85. Allow user to accept/reject/modify suggestions

---

## 7. Phase 5: Data Mapping

**Estimated Duration: 4-5 weeks**

### 7.1 Source-to-Model Mapping

86. Create mapping panel UI linked to model elements
87. Implement source (table/file) selection for nodes/relationships
88. Build column-to-property mapping interface
89. Add automatic mapping suggestions based on column names
90. Support mapping multiple sources to same node type
91. Implement SQL query-based mapping for complex extractions

### 7.2 ID and Reference Mapping

92. Implement ID column selection for nodes
93. Support composite ID keys (multiple columns)
94. Create FROM/TO column mapping for relationships
95. Add ID transformation options (prefix, suffix, concatenation, hash)
96. Validate ID uniqueness during preview
97. Support foreign key-based relationship detection

### 7.3 Data Transformation

98. Implement type conversion functions (toInteger, toFloat, toBoolean, toDate)
99. Add date format configuration with common presets
100. Support null value handling (skip, default value, empty string)
101. Add value trimming and normalization options
102. Implement array splitting from delimited strings
103. Add string manipulation (substring, replace, concat)
104. Support computed/derived properties

### 7.4 Data Filtering

105. Add row filtering by column value
106. Implement condition builder (equals, not equals, contains, regex, range)
107. Support combining multiple filters with AND/OR
108. Add SQL WHERE clause support for database sources
109. Implement sampling for large datasets

---

## 8. Phase 6: Import Engine (Rust)

**Estimated Duration: 5-6 weeks**

### 8.1 Cypher Query Generation

110. Build Cypher query generator from mapping configuration
111. Generate MERGE statements for nodes with ON CREATE/ON MATCH
112. Generate MERGE statements for relationships
113. Implement parameterized queries with UNWIND for batch processing
114. Add query preview with syntax highlighting
115. Support CREATE vs MERGE mode selection
116. Generate optimized queries for large batches

### 8.2 Parallel Import Pipeline

117. Implement async data streaming from sources (Tokio streams)
118. Create parallel worker pool for import execution
119. Implement backpressure handling for memory management
120. Add configurable parallelism (worker count, batch size)
121. Create data transformation pipeline
122. Implement buffered batching for optimal throughput

### 8.3 Import Execution

123. Implement batch import with configurable batch size
124. Add transaction management (commit per batch)
125. Create progress tracking with estimated time remaining
126. Implement WebSocket-based real-time progress updates
127. Add import cancellation with graceful cleanup
128. Implement pause/resume capability with checkpointing
129. Add dry-run mode for validation without import

### 8.4 Error Handling

130. Implement error collection during import
131. Add error categorization (data type, constraint violation, connection)
132. Create error report with problematic rows and source context
133. Implement skip-on-error option with threshold
134. Add error export to CSV
135. Implement retry logic for transient failures
136. Add dead letter queue for failed records

### 8.5 Indexes and Constraints

137. Implement index creation UI for node properties
138. Add unique constraint configuration
139. Generate Cypher for index/constraint creation
140. Execute index creation before data import
141. Support full-text index configuration

---

## 9. Phase 7: Configuration Management

**Estimated Duration: 2-3 weeks**

### 9.1 Configuration Schema

142. Define JSON schema for import configuration
143. Include data source connection details (encrypted)
144. Include model definition (nodes, relationships, properties)
145. Include source mappings and transformations
146. Version the configuration schema for future compatibility

### 9.2 Save/Load Functionality

147. Implement configuration export to JSON file
148. Add configuration import from JSON file
149. Support configuration with or without connection credentials
150. Add validation on configuration load
151. Implement browser local storage autosave
152. Add configuration versioning and migration

### 9.3 Template Library

153. Create starter templates for common use cases:
    - E-commerce (products, customers, orders)
    - Social network (users, posts, follows)
    - Knowledge graph (entities, relationships)
154. Add template browser UI
155. Support user-saved templates
156. Add template sharing/export

---

## 10. Phase 8: Polish, Testing & Documentation

**Estimated Duration: 3-4 weeks**

### 10.1 User Experience

157. Add keyboard shortcuts for common actions
158. Implement guided onboarding/tutorial
159. Add contextual help tooltips
160. Implement validation feedback throughout UI
161. Add dark mode support
162. Implement responsive design for tablet/desktop

### 10.2 Documentation

163. Write user documentation with tutorials
164. Create API documentation (OpenAPI/Swagger)
165. Document configuration schema with examples
166. Add inline help in the application
167. Create video walkthroughs
168. Document connector-specific requirements

### 10.3 Testing & Quality

169. Write unit tests for Rust connectors and parsers
170. Write unit tests for Cypher query generator
171. Add integration tests for each data source connector
172. Perform end-to-end testing with sample datasets
173. Load testing with large datasets (10M+ rows)
174. Stress testing for concurrent imports
175. Cross-browser compatibility testing
176. Security testing (SQL injection, credential handling)

---

## 11. Mapping Configuration Schema

The following JSON schema defines the structure for import configurations:

```json
{
  "version": "1.0",
  "name": "My Import Configuration",
  "description": "Import from PostgreSQL to FalkorDB",
  
  "source": {
    "type": "postgresql",
    "config": {
      "host": "db.example.com",
      "port": 5432,
      "database": "mydb",
      "schema": "public",
      "username": "user",
      "password": "encrypted:...",
      "ssl": true
    }
  },
  
  "target": {
    "type": "falkordb",
    "config": {
      "host": "localhost",
      "port": 6379,
      "graph": "mygraph",
      "username": "default",
      "password": "encrypted:..."
    }
  },
  
  "model": {
    "nodes": [{
      "id": "node-person",
      "labels": ["Person"],
      "properties": [{
        "name": "id",
        "type": "string",
        "isId": true
      }, {
        "name": "name",
        "type": "string"
      }, {
        "name": "age",
        "type": "integer"
      }],
      "position": { "x": 100, "y": 100 }
    }, {
      "id": "node-movie",
      "labels": ["Movie"],
      "properties": [{
        "name": "id",
        "type": "string",
        "isId": true
      }, {
        "name": "title",
        "type": "string"
      }, {
        "name": "year",
        "type": "integer"
      }],
      "position": { "x": 400, "y": 100 }
    }],
    "relationships": [{
      "id": "rel-acted-in",
      "type": "ACTED_IN",
      "fromNodeId": "node-person",
      "toNodeId": "node-movie",
      "properties": [{
        "name": "role",
        "type": "string"
      }]
    }]
  },
  
  "mappings": [{
    "elementId": "node-person",
    "source": {
      "type": "table",
      "table": "actors",
      "query": null
    },
    "columnMappings": [{
      "column": "actor_id",
      "property": "id",
      "transform": null
    }, {
      "column": "full_name",
      "property": "name",
      "transform": { "type": "trim" }
    }, {
      "column": "birth_year",
      "property": "age",
      "transform": { "type": "toInteger" }
    }],
    "filter": null
  }, {
    "elementId": "node-movie",
    "source": {
      "type": "query",
      "table": null,
      "query": "SELECT movie_id, title, release_year FROM movies WHERE active = true"
    },
    "columnMappings": [{
      "column": "movie_id",
      "property": "id",
      "transform": null
    }, {
      "column": "title",
      "property": "title",
      "transform": null
    }, {
      "column": "release_year",
      "property": "year",
      "transform": { "type": "toInteger" }
    }],
    "filter": null
  }, {
    "elementId": "rel-acted-in",
    "source": {
      "type": "table",
      "table": "movie_cast"
    },
    "fromColumn": "actor_id",
    "toColumn": "movie_id",
    "columnMappings": [{
      "column": "character_name",
      "property": "role",
      "transform": null
    }],
    "filter": {
      "conditions": [{
        "column": "role_type",
        "operator": "equals",
        "value": "ACTOR"
      }],
      "logic": "AND"
    }
  }],
  
  "indexes": [{
    "nodeLabel": "Person",
    "property": "id",
    "type": "unique"
  }, {
    "nodeLabel": "Movie",
    "property": "id",
    "type": "unique"
  }, {
    "nodeLabel": "Person",
    "property": "name",
    "type": "fulltext"
  }],
  
  "settings": {
    "batchSize": 5000,
    "parallelism": 4,
    "skipOnError": false,
    "errorThreshold": 100,
    "createIndexesFirst": true,
    "mode": "merge"
  }
}
```

### Supported Source Types

| Type | Configuration Fields |
|------|---------------------|
| `postgresql` | host, port, database, schema, username, password, ssl |
| `mysql` | host, port, database, username, password, ssl |
| `sqlserver` | host, port, database, username, password, trustServerCertificate |
| `oracle` | host, port, service, username, password |
| `snowflake` | account, warehouse, database, schema, username, password/privateKey |
| `bigquery` | project, dataset, credentialsJson |
| `databricks` | host, token, httpPath, catalog, schema |
| `s3` | region, bucket, prefix, accessKeyId, secretAccessKey |
| `azure_blob` | connectionString, container, prefix |
| `gcs` | project, bucket, prefix, credentialsJson |
| `csv` | (file uploaded separately) delimiter, hasHeader, encoding |
| `json` | (file uploaded separately) rootPath, flatten |
| `parquet` | (file uploaded separately) |

### Supported Transforms

| Transform | Description |
|-----------|-------------|
| `trim` | Remove leading/trailing whitespace |
| `toInteger` | Convert to integer |
| `toFloat` | Convert to floating point |
| `toBoolean` | Convert to boolean (true/false) |
| `toDate` | Parse as date (with format option) |
| `toDateTime` | Parse as datetime |
| `split(delimiter)` | Split string into array |
| `uppercase` | Convert to uppercase |
| `lowercase` | Convert to lowercase |
| `substring(start, end)` | Extract substring |
| `replace(from, to)` | Replace string |
| `concat(fields...)` | Concatenate multiple fields |
| `hash(algorithm)` | Generate hash (md5, sha256) |
| `default(value)` | Use default if null |

---

## 12. Timeline Summary

| Phase | Duration | Key Deliverables |
|-------|----------|------------------|
| Phase 1: Foundation | 4-5 weeks | Rust backend, Axum API, connector traits |
| Phase 2: Data Source Connectors | 5-6 weeks | All database and storage connectors |
| Phase 3: Frontend & Source UI | 4-5 weeks | Connection UI, schema browser, file upload |
| Phase 4: Visual Modeling | 4-5 weeks | Graph canvas, node/rel configuration |
| Phase 5: Data Mapping | 4-5 weeks | Mapping UI, transformations, filtering |
| Phase 6: Import Engine | 5-6 weeks | Cypher generation, parallel import, error handling |
| Phase 7: Configuration | 2-3 weeks | Save/load configs, templates |
| Phase 8: Polish & Testing | 3-4 weeks | UX, documentation, testing |

**Total Estimated Duration: 32-39 weeks (approximately 8-10 months)**

---

## 13. Success Metrics

- Import 10M+ rows from PostgreSQL in under 10 minutes
- Support concurrent connections to 5+ different source types
- Parallel import with 4+ workers for maximum throughput
- Memory usage under 2GB for 1M row imports
- Error rate less than 0.1% on well-formed data
- API response time under 100ms for schema discovery
- WebSocket progress updates with <500ms latency
- Configuration files fully compatible across versions

---

## 14. Technical Stack

### 14.1 Backend (Rust)

| Component | Crate/Tool |
|-----------|------------|
| Web Framework | Axum |
| Async Runtime | Tokio |
| PostgreSQL | sqlx or tokio-postgres |
| MySQL | mysql_async |
| SQL Server | tiberius |
| Oracle | oracle or ODBC |
| Snowflake | snowflake-api or REST |
| BigQuery | gcp-bigquery-client |
| AWS S3 | aws-sdk-s3 |
| Azure Blob | azure_storage_blobs |
| GCS | cloud-storage |
| CSV | csv |
| JSON | serde_json |
| Parquet | parquet |
| Excel | calamine |
| Serialization | serde |
| Error Handling | thiserror, anyhow |
| Logging | tracing |
| Config | config |
| OpenAPI | utoipa |
| WebSocket | axum + tokio-tungstenite |

### 14.2 Frontend (TypeScript)

| Component | Library |
|-----------|---------|
| Framework | React 18+ |
| Build Tool | Vite |
| State Management | Zustand |
| Graph Visualization | React Flow |
| UI Components | Radix UI + Tailwind CSS |
| Forms | React Hook Form + Zod |
| HTTP Client | TanStack Query + fetch |
| WebSocket | native WebSocket |
| Code Editor | Monaco Editor (for SQL) |

### 14.3 Infrastructure

| Component | Technology |
|-----------|------------|
| Container | Docker |
| Orchestration | Docker Compose / Kubernetes |
| CI/CD | GitHub Actions |
| Testing | Rust: cargo test, Jest for frontend |
| Load Testing | k6 or Locust |

---

## 15. Rust Connector Trait Definition

```rust
use async_trait::async_trait;
use tokio_stream::Stream;
use std::pin::Pin;

#[async_trait]
pub trait DataSourceConnector: Send + Sync {
    /// Test the connection with provided credentials
    async fn test_connection(&self) -> Result<(), ConnectorError>;
    
    /// Discover available schemas/databases
    async fn discover_schemas(&self) -> Result<Vec<SchemaInfo>, ConnectorError>;
    
    /// List tables/collections in a schema
    async fn list_tables(&self, schema: &str) -> Result<Vec<TableInfo>, ConnectorError>;
    
    /// Get detailed schema for a specific table
    async fn get_table_schema(&self, schema: &str, table: &str) 
        -> Result<TableSchema, ConnectorError>;
    
    /// Preview data from a table (limited rows)
    async fn preview_data(&self, schema: &str, table: &str, limit: usize) 
        -> Result<Vec<Row>, ConnectorError>;
    
    /// Execute a custom query and return results
    async fn execute_query(&self, query: &str) 
        -> Result<Vec<Row>, ConnectorError>;
    
    /// Stream data from a table or query for large datasets
    fn stream_data(&self, source: DataSource) 
        -> Pin<Box<dyn Stream<Item = Result<Row, ConnectorError>> + Send>>;
    
    /// Get estimated row count for progress tracking
    async fn estimate_row_count(&self, source: &DataSource) 
        -> Result<Option<u64>, ConnectorError>;
}

#[derive(Debug, Clone)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub primary_key: Option<Vec<String>>,
    pub foreign_keys: Vec<ForeignKeyInfo>,
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

#[derive(Debug, Clone)]
pub enum DataType {
    String,
    Integer,
    Float,
    Boolean,
    Date,
    DateTime,
    Binary,
    Json,
    Array(Box<DataType>),
    Unknown(String),
}
```

---

## 16. Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Connector complexity | Start with PostgreSQL and CSV; add others incrementally |
| Large dataset performance | Implement streaming, backpressure, parallel workers |
| Cloud credential security | Encrypt at rest, never log credentials, short-lived tokens |
| Database driver compatibility | Extensive testing matrix; fallback to ODBC |
| Memory exhaustion | Streaming architecture, configurable batch sizes |
| Network failures | Retry logic, checkpoint/resume for long imports |
| Schema changes | Versioned config format with migration support |

---

## 17. Future Enhancements (Post-MVP)

- Scheduled/recurring imports with cron expressions
- Incremental import (CDC - Change Data Capture)
- Real-time streaming import (Kafka, Kinesis)
- GraphQL API for programmatic imports
- Import history and audit logging
- Collaborative model editing
- AI-assisted model suggestions from schema
- Data quality profiling and validation rules
- Multi-graph target support
- Export from FalkorDB to other systems
- Kubernetes operator for managed deployments
