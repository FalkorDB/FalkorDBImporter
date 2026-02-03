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
- Import from cloud storage (AWS S3, Azure Blob/Data Lake, Google Cloud Storage)# FalkorDB Data Importer
## Project Plan & Task Breakdown

**February 2026**

---

## 1. Executive Summary

This document outlines the project plan for building a Data Importer service for FalkorDB, inspired by Neo4j's Aura Import service. The service will provide a high-performance, scalable tool for importing data from multiple sources into FalkorDB, with visual data modeling and mapping capabilities.

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

The architecture leverages FalkorDB's native `LOAD CSV` capability for optimal performance. The Rust server acts as a data bridge, fetching data from external sources and exposing it as CSV via HTTP endpoints that FalkorDB can consume directly.

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Frontend (React/TypeScript)                   │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │ Source   │ │ Schema   │ │ Model    │ │ Mapping  │ │ Import   │  │
│  │ Connect  │ │ Browser  │ │ Canvas   │ │ Panel    │ │ Runner   │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                                   │
                            REST API (config, schema, status)
                                   │
┌─────────────────────────────────────────────────────────────────────┐
│                      Rust Backend (Axum)                             │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │                    Data Source Connectors                       │ │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐  │ │
│  │  │Postgres │ │ MySQL   │ │Snowflake│ │  S3     │ │  File   │  │ │
│  │  │Connector│ │Connector│ │Connector│ │Connector│ │ Upload  │  │ │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘  │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                │                                     │
│                         Data Extraction                              │
│                                │                                     │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │              CSV HTTP Endpoint Server                           │ │
│  │                                                                  │ │
│  │   GET /data/{job_id}/nodes/{label}.csv                          │ │
│  │   GET /data/{job_id}/edges/{type}.csv                           │ │
│  │                                                                  │ │
│  │   - Streams data as CSV with proper headers                     │ │
│  │   - Applies column mappings and transformations                 │ │
│  │   - Handles pagination for large datasets                       │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │              Cypher Query Generator                             │ │
│  │                                                                  │ │
│  │   Generates LOAD CSV queries pointing to HTTP endpoints:        │ │
│  │                                                                  │ │
│  │   LOAD CSV WITH HEADERS FROM 'http://server/data/123/nodes/     │ │
│  │     Person.csv' AS row                                          │ │
│  │   MERGE (p:Person {id: row.id})                                 │ │
│  │   SET p.name = row.name, p.age = toInteger(row.age)             │ │
│  └────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
                                   │
                          LOAD CSV via HTTP
                                   │
                         ┌─────────────────────┐
                         │     FalkorDB        │
                         │                     │
                         │  LOAD CSV FROM      │
                         │  'http://...'       │
                         └─────────────────────┘
```

### 2.4 Import Flow

1. **User configures import** → Connects to source, maps columns to graph model
2. **User triggers import** → Backend creates a job with unique ID
3. **Backend prepares data** → Extracts data from source, applies transformations
4. **Backend exposes CSV endpoints** → `http://server/data/{job_id}/nodes/Person.csv`
5. **Backend generates Cypher** → `LOAD CSV WITH HEADERS FROM 'http://...' AS row MERGE ...`
6. **Backend executes Cypher on FalkorDB** → FalkorDB fetches CSV via HTTP and loads data
7. **Progress tracking** → Backend monitors query execution, reports to frontend

### 2.5 Why This Architecture?

| Benefit | Description |
|---------|-------------|
| **Leverages FalkorDB's optimized LOAD CSV** | Native bulk loading is faster than individual inserts |
| **Lower memory on Rust server** | Streams data on-demand rather than holding in memory |
| **Simpler Rust code** | No need for FalkorDB client bindings; just HTTP + SQL clients |
| **Scalable** | CSV endpoints can be cached, load-balanced, or served from CDN |
| **Debuggable** | CSV files can be inspected directly via browser |
| **Supports both local and cloud** | Same pattern works for FalkorDB Cloud (HTTPS) and local (HTTP) |

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

### 3.3 CSV HTTP Endpoint Server

15. Create job management system (create job, track status, cleanup)
16. Implement CSV streaming endpoint: `GET /data/{job_id}/nodes/{label}.csv`
17. Implement CSV streaming endpoint: `GET /data/{job_id}/edges/{type}.csv`
18. Add CSV header generation from mapping configuration
19. Implement data transformation pipeline (type conversion, trimming, etc.)
20. Add streaming response with proper `Content-Type: text/csv` headers
21. Implement job expiration and cleanup (TTL-based)
22. Add authentication/token validation for CSV endpoints

### 3.4 Connector Trait Architecture

23. Define `DataSourceConnector` trait with async methods:
    - `connect()` - Establish connection
    - `test_connection()` - Validate credentials
    - `discover_schema()` - List tables/collections
    - `get_table_schema()` - Get columns and types
    - `preview_data()` - Fetch sample rows
    - `stream_data()` - Async data streaming iterator
24. Define `DataSourceConfig` enum for connection parameters
25. Create connector registry for dynamic connector loading
26. Implement connection pooling abstraction

### 3.5 FalkorDB Query Execution

27. Implement FalkorDB connection via Redis protocol (redis crate)
28. Create Cypher query executor using GRAPH.QUERY command
29. Add query result parsing
30. Implement connection testing for FalkorDB target

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

## 8. Phase 6: Import Engine (LOAD CSV Based)

**Estimated Duration: 4-5 weeks**

### 8.1 Cypher LOAD CSV Query Generation

110. Build Cypher query generator from mapping configuration
111. Generate `LOAD CSV WITH HEADERS FROM` statements with HTTP URLs
112. Generate node creation queries with MERGE/CREATE:
     ```cypher
     LOAD CSV WITH HEADERS FROM 'http://server/data/{job}/nodes/Person.csv' AS row
     MERGE (p:Person {id: row.id})
     SET p.name = row.name, p.age = toInteger(row.age)
     ```
113. Generate relationship creation queries:
     ```cypher
     LOAD CSV WITH HEADERS FROM 'http://server/data/{job}/edges/ACTED_IN.csv' AS row
     MATCH (a:Person {id: row.from_id})
     MATCH (m:Movie {id: row.to_id})
     MERGE (a)-[r:ACTED_IN]->(m)
     SET r.role = row.role
     ```
114. Add query preview with syntax highlighting
115. Support CREATE vs MERGE mode selection
116. Handle type conversions in Cypher (toInteger, toFloat, toBoolean, date)

### 8.2 CSV Data Preparation Pipeline

117. Implement async data extraction from source connectors
118. Create transformation pipeline:
    - Column selection and renaming
    - Type conversion (dates, numbers, booleans)
    - Null handling (skip, default value)
    - String manipulation (trim, case conversion)
119. Generate separate CSV streams for each node label
120. Generate separate CSV streams for each relationship type
121. Add computed columns for relationship from/to IDs
122. Implement CSV escaping and quoting (RFC 4180 compliant)

### 8.3 Import Execution Orchestration

123. Create import job state machine (pending → preparing → importing → complete/failed)
124. Execute index/constraint creation queries first
125. Execute node LOAD CSV queries (in dependency order)
126. Execute relationship LOAD CSV queries (after nodes exist)
127. Implement query execution via FalkorDB GRAPH.QUERY command
128. Parse query results and statistics (nodes created, relationships created)

### 8.4 Progress Tracking

129. Track job progress: data preparation %, nodes loaded %, edges loaded %
130. Implement WebSocket endpoint for real-time progress updates
131. Calculate estimated time remaining based on throughput
132. Add import cancellation with cleanup
133. Implement job history and status persistence

### 8.5 Error Handling

134. Capture and categorize errors (connection, data type, constraint violation)
135. Implement skip-on-error mode with error threshold
136. Create error report with failed rows and reasons
137. Add error export to CSV
138. Implement retry logic for transient failures

### 8.6 Indexes and Constraints

139. Implement index creation UI for node properties
140. Add unique constraint configuration
141. Generate Cypher for index/constraint creation:
     ```cypher
     CREATE INDEX FOR (p:Person) ON (p.id)
     CREATE CONSTRAINT FOR (p:Person) REQUIRE p.id IS UNIQUE
     ```
142. Execute index creation before data import
143. Support full-text index configuration

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
| Phase 1: Foundation | 4-5 weeks | Rust backend, CSV HTTP endpoints, connector traits |
| Phase 2: Data Source Connectors | 5-6 weeks | All database and storage connectors |
| Phase 3: Frontend & Source UI | 4-5 weeks | Connection UI, schema browser, file upload |
| Phase 4: Visual Modeling | 4-5 weeks | Graph canvas, node/rel configuration |
| Phase 5: Data Mapping | 4-5 weeks | Mapping UI, transformations, filtering |
| Phase 6: Import Engine | 4-5 weeks | LOAD CSV generation, execution, progress tracking |
| Phase 7: Configuration | 2-3 weeks | Save/load configs, templates |
| Phase 8: Polish & Testing | 3-4 weeks | UX, documentation, testing |

**Total Estimated Duration: 31-38 weeks (approximately 7-9 months)**

---

## 13. Success Metrics

- Import 10M+ rows from PostgreSQL in under 10 minutes using LOAD CSV
- Support concurrent connections to 5+ different source types
- CSV endpoint response time under 50ms for first byte (streaming)
- Memory usage under 500MB for 10M row imports (streaming architecture)
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

## 16. Example Generated LOAD CSV Queries

The import engine generates Cypher queries that use FalkorDB's `LOAD CSV` to fetch data from the Rust server's HTTP endpoints.

### 16.1 Node Import Example

**CSV Endpoint:** `GET http://importer:8080/data/job-123/nodes/Person.csv`

```csv
id,name,age,email
p001,Alice Smith,32,alice@example.com
p002,Bob Jones,28,bob@example.com
p003,Carol White,45,carol@example.com
```

**Generated Cypher:**

```cypher
// Create index first for fast lookups
CREATE INDEX FOR (p:Person) ON (p.id)
```

```cypher
// Load nodes
LOAD CSV WITH HEADERS FROM 'http://importer:8080/data/job-123/nodes/Person.csv' AS row
MERGE (p:Person {id: row.id})
SET p.name = row.name,
    p.age = toInteger(row.age),
    p.email = row.email
```

### 16.2 Relationship Import Example

**CSV Endpoint:** `GET http://importer:8080/data/job-123/edges/FOLLOWS.csv`

```csv
from_id,to_id,since,weight
p001,p002,2020-01-15,0.8
p002,p003,2021-06-20,0.5
p001,p003,2019-03-10,0.9
```

**Generated Cypher:**

```cypher
LOAD CSV WITH HEADERS FROM 'http://importer:8080/data/job-123/edges/FOLLOWS.csv' AS row
MATCH (a:Person {id: row.from_id})
MATCH (b:Person {id: row.to_id})
MERGE (a)-[r:FOLLOWS]->(b)
SET r.since = date(row.since),
    r.weight = toFloat(row.weight)
```

### 16.3 Full Import Sequence

For a complete import job, the engine generates and executes queries in this order:

```cypher
// 1. Create indexes and constraints
CREATE INDEX FOR (p:Person) ON (p.id)
CREATE INDEX FOR (m:Movie) ON (m.id)
CREATE CONSTRAINT FOR (p:Person) REQUIRE p.id IS UNIQUE

// 2. Load all node types
LOAD CSV WITH HEADERS FROM 'http://importer:8080/data/job-123/nodes/Person.csv' AS row
MERGE (p:Person {id: row.id})
SET p.name = row.name, p.age = toInteger(row.age)

LOAD CSV WITH HEADERS FROM 'http://importer:8080/data/job-123/nodes/Movie.csv' AS row
MERGE (m:Movie {id: row.id})
SET m.title = row.title, m.year = toInteger(row.year)

// 3. Load all relationship types (after nodes exist)
LOAD CSV WITH HEADERS FROM 'http://importer:8080/data/job-123/edges/ACTED_IN.csv' AS row
MATCH (p:Person {id: row.person_id})
MATCH (m:Movie {id: row.movie_id})
MERGE (p)-[r:ACTED_IN]->(m)
SET r.role = row.role

LOAD CSV WITH HEADERS FROM 'http://importer:8080/data/job-123/edges/DIRECTED.csv' AS row
MATCH (p:Person {id: row.person_id})
MATCH (m:Movie {id: row.movie_id})
MERGE (p)-[r:DIRECTED]->(m)
```

### 16.4 Type Conversion Functions

FalkorDB supports these type conversions in LOAD CSV queries:

| Function | Description | Example |
|----------|-------------|---------|
| `toInteger(x)` | Convert to integer | `toInteger(row.age)` |
| `toFloat(x)` | Convert to float | `toFloat(row.price)` |
| `toBoolean(x)` | Convert to boolean | `toBoolean(row.active)` |
| `date(x)` | Parse ISO date | `date(row.birth_date)` |
| `datetime(x)` | Parse ISO datetime | `datetime(row.created_at)` |
| `split(x, delim)` | Split to array | `split(row.tags, ',')` |

---

## 17. Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Connector complexity | Start with PostgreSQL and CSV; add others incrementally |
| Large dataset performance | Streaming CSV endpoints, FalkorDB's optimized LOAD CSV |
| Cloud credential security | Encrypt at rest, never log credentials, short-lived tokens |
| Database driver compatibility | Extensive testing matrix; fallback to ODBC |
| Memory exhaustion | Streaming architecture - data flows through, not stored |
| Network failures | Retry logic, job checkpointing for resume |
| CSV endpoint availability | Job TTL with cleanup; token-based access control |
| FalkorDB connectivity | Connection testing before import; clear error messages |

---

## 18. Future Enhancements (Post-MVP)

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
- Direct S3/GCS URLs for LOAD CSV (when FalkorDB supports it)
