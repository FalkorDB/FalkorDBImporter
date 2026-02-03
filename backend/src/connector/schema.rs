use serde::{Deserialize, Serialize};

/// Information about a table or collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    /// Name of the table/collection
    pub name: String,

    /// Schema or database name (if applicable)
    pub schema: Option<String>,

    /// Type of table (table, view, materialized view, etc.)
    pub table_type: TableType,

    /// Columns in the table
    pub columns: Vec<ColumnInfo>,

    /// Primary key columns
    pub primary_keys: Vec<String>,

    /// Foreign key relationships
    pub foreign_keys: Vec<ForeignKeyInfo>,

    /// Estimated row count (if available)
    pub row_count: Option<u64>,
}

/// Type of table or view
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TableType {
    Table,
    View,
    MaterializedView,
    Collection,
    Other(String),
}

/// Information about a column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    /// Column name
    pub name: String,

    /// Data type
    pub data_type: crate::connector::types::DataType,

    /// Whether the column is nullable
    pub nullable: bool,

    /// Whether the column is part of primary key
    pub is_primary_key: bool,

    /// Default value (if any)
    pub default_value: Option<String>,

    /// Maximum length for string types
    pub max_length: Option<u32>,

    /// Precision for numeric types
    pub precision: Option<u32>,

    /// Scale for numeric types
    pub scale: Option<u32>,
}

/// Foreign key relationship information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyInfo {
    /// Name of the foreign key constraint
    pub name: String,

    /// Column(s) in this table
    pub columns: Vec<String>,

    /// Referenced table name
    pub referenced_table: String,

    /// Referenced column(s)
    pub referenced_columns: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::types::DataType;

    #[test]
    fn test_table_info_creation() {
        let table = TableInfo {
            name: "users".to_string(),
            schema: Some("public".to_string()),
            table_type: TableType::Table,
            columns: vec![],
            primary_keys: vec!["id".to_string()],
            foreign_keys: vec![],
            row_count: Some(100),
        };
        assert_eq!(table.name, "users");
        assert_eq!(table.schema, Some("public".to_string()));
        assert_eq!(table.table_type, TableType::Table);
    }

    #[test]
    fn test_column_info_creation() {
        let column = ColumnInfo {
            name: "id".to_string(),
            data_type: DataType::Integer,
            nullable: false,
            is_primary_key: true,
            default_value: None,
            max_length: None,
            precision: None,
            scale: None,
        };
        assert_eq!(column.name, "id");
        assert_eq!(column.data_type, DataType::Integer);
        assert!(!column.nullable);
        assert!(column.is_primary_key);
    }
}
