use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Data type for columns and values
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    // Numeric types
    Integer,
    BigInt,
    SmallInt,
    Float,
    Double,
    Decimal,
    Numeric,

    // String types
    String,
    Text,
    Varchar,
    Char,

    // Binary types
    Binary,
    Blob,

    // Date/Time types
    Date,
    Time,
    DateTime,
    Timestamp,
    TimestampTz,

    // Boolean type
    Boolean,

    // JSON types
    Json,
    JsonB,

    // Array types
    Array(Box<DataType>),

    // Complex types
    Uuid,
    Xml,
    Inet,
    Cidr,

    // Unknown/Other
    Unknown,
    Other(String),
}

/// A single data value
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum DataValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Binary(Vec<u8>),
    Array(Vec<DataValue>),
    Object(HashMap<String, DataValue>),
}

impl DataValue {
    /// Convert to string representation
    #[allow(dead_code)]
    pub fn to_string_value(&self) -> String {
        match self {
            DataValue::Null => String::new(),
            DataValue::Boolean(b) => b.to_string(),
            DataValue::Integer(i) => i.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::String(s) => s.clone(),
            DataValue::Binary(b) => format!("0x{}", hex::encode(b)),
            DataValue::Array(arr) => format!(
                "[{}]",
                arr.iter()
                    .map(|v| v.to_string_value())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            DataValue::Object(_) => serde_json::to_string(self).unwrap_or_default(),
        }
    }

    /// Check if value is null
    #[allow(dead_code)]
    pub fn is_null(&self) -> bool {
        matches!(self, DataValue::Null)
    }
}

/// A single row of data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRow {
    /// Column values indexed by column name
    pub values: HashMap<String, DataValue>,
}

impl DataRow {
    /// Create a new empty data row
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Create a data row with initial values
    #[allow(dead_code)]
    pub fn with_values(values: HashMap<String, DataValue>) -> Self {
        Self { values }
    }

    /// Get a value by column name
    #[allow(dead_code)]
    pub fn get(&self, column: &str) -> Option<&DataValue> {
        self.values.get(column)
    }

    /// Set a value for a column
    #[allow(dead_code)]
    pub fn set(&mut self, column: String, value: DataValue) {
        self.values.insert(column, value);
    }

    /// Get all column names
    #[allow(dead_code)]
    pub fn columns(&self) -> Vec<&String> {
        self.values.keys().collect()
    }
}

impl Default for DataRow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_value_to_string() {
        assert_eq!(DataValue::Null.to_string_value(), "");
        assert_eq!(DataValue::Boolean(true).to_string_value(), "true");
        assert_eq!(DataValue::Integer(42).to_string_value(), "42");
        assert_eq!(DataValue::Float(2.5).to_string_value(), "2.5");
        assert_eq!(
            DataValue::String("hello".to_string()).to_string_value(),
            "hello"
        );
    }

    #[test]
    fn test_data_value_is_null() {
        assert!(DataValue::Null.is_null());
        assert!(!DataValue::Integer(0).is_null());
        assert!(!DataValue::String("".to_string()).is_null());
    }

    #[test]
    fn test_data_row_operations() {
        let mut row = DataRow::new();
        row.set("id".to_string(), DataValue::Integer(1));
        row.set("name".to_string(), DataValue::String("Alice".to_string()));

        assert_eq!(row.get("id"), Some(&DataValue::Integer(1)));
        assert_eq!(
            row.get("name"),
            Some(&DataValue::String("Alice".to_string()))
        );
        assert_eq!(row.get("age"), None);
        assert_eq!(row.columns().len(), 2);
    }

    #[test]
    fn test_data_row_with_values() {
        let mut values = HashMap::new();
        values.insert("id".to_string(), DataValue::Integer(1));
        values.insert("active".to_string(), DataValue::Boolean(true));

        let row = DataRow::with_values(values);
        assert_eq!(row.get("id"), Some(&DataValue::Integer(1)));
        assert_eq!(row.get("active"), Some(&DataValue::Boolean(true)));
    }

    #[test]
    fn test_data_type_array() {
        let arr_type = DataType::Array(Box::new(DataType::Integer));
        match arr_type {
            DataType::Array(inner) => assert_eq!(*inner, DataType::Integer),
            _ => panic!("Expected Array type"),
        }
    }
}
