use crate::error::{AppError, AppResult};
use crate::jobs::{JobManager, NodeMapping, EdgeMapping, ColumnMapping};
use axum::{
    extract::{Path, Request, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use std::sync::Arc;

/// Data transformation utilities
pub struct DataTransformer;

impl DataTransformer {
    /// Transform a value according to column mapping
    pub fn transform_value(
        value: &serde_json::Value,
        column: &ColumnMapping,
    ) -> String {
        let mut result = match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => String::new(),
            _ => value.to_string(),
        };

        // Apply trimming if configured
        if column.trim {
            result = result.trim().to_string();
        }

        // Apply type conversion if specified
        if let Some(data_type) = &column.data_type {
            result = Self::convert_type(&result, data_type);
        }

        result
    }

    /// Convert value to specified data type
    fn convert_type(value: &str, data_type: &str) -> String {
        match data_type.to_lowercase().as_str() {
            "integer" | "int" => {
                value.parse::<i64>().map(|v| v.to_string()).unwrap_or_default()
            }
            "float" | "double" => {
                value.parse::<f64>().map(|v| v.to_string()).unwrap_or_default()
            }
            "boolean" | "bool" => {
                value.parse::<bool>().map(|v| v.to_string()).unwrap_or_default()
            }
            _ => value.to_string(),
        }
    }

    /// Generate CSV header from column mappings
    pub fn generate_csv_header(columns: &[ColumnMapping]) -> String {
        columns
            .iter()
            .map(|col| Self::escape_csv_field(&col.target_property))
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Generate CSV row from data and column mappings
    pub fn generate_csv_row(
        data: &std::collections::HashMap<String, serde_json::Value>,
        columns: &[ColumnMapping],
    ) -> String {
        columns
            .iter()
            .map(|col| {
                let value = data
                    .get(&col.source_column)
                    .unwrap_or(&serde_json::Value::Null);
                let transformed = Self::transform_value(value, col);
                Self::escape_csv_field(&transformed)
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Escape CSV field (handle quotes and commas)
    fn escape_csv_field(field: &str) -> String {
        if field.contains(',') || field.contains('"') || field.contains('\n') {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }
}

/// Validate authentication token for a job
fn validate_auth(
    job: &crate::jobs::Job,
    request: &Request,
) -> AppResult<()> {
    // If job has an auth token, validate it
    if let Some(expected_token) = &job.auth_token {
        let auth_header = request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::BadRequest("Missing Authorization header".to_string()))?;

        let provided_token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::BadRequest("Invalid Authorization header format".to_string()))?;

        if provided_token != expected_token {
            return Err(AppError::BadRequest("Invalid authentication token".to_string()));
        }
    }
    Ok(())
}

/// Serve CSV data for nodes
#[utoipa::path(
    get,
    path = "/data/{job_id}/nodes/{label}",
    params(
        ("job_id" = String, Path, description = "Job ID"),
        ("label" = String, Path, description = "Node label")
    ),
    responses(
        (status = 200, description = "CSV data stream", content_type = "text/csv"),
        (status = 404, description = "Job or node label not found"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "csv"
)]
pub async fn serve_node_csv(
    State(job_manager): State<Arc<JobManager>>,
    Path((job_id, label)): Path<(String, String)>,
    request: Request,
) -> AppResult<Response> {
    tracing::debug!("Serving node CSV for job: {}, label: {}", job_id, label);

    // Get the job
    let job = job_manager.get_job(&job_id).await?;

    // Validate authentication
    validate_auth(&job, &request)?;

    // Find the node mapping
    let node_mapping = job
        .mapping
        .nodes
        .iter()
        .find(|n| n.label == label)
        .ok_or_else(|| AppError::NotFound(format!("Node label not found: {}", label)))?;

    // Generate CSV content
    let csv_content = generate_node_csv(node_mapping)?;

    // Return response with proper headers
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/csv; charset=utf-8")],
        csv_content,
    )
        .into_response())
}

/// Serve CSV data for edges
#[utoipa::path(
    get,
    path = "/data/{job_id}/edges/{edge_type}",
    params(
        ("job_id" = String, Path, description = "Job ID"),
        ("edge_type" = String, Path, description = "Edge type")
    ),
    responses(
        (status = 200, description = "CSV data stream", content_type = "text/csv"),
        (status = 404, description = "Job or edge type not found"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "csv"
)]
pub async fn serve_edge_csv(
    State(job_manager): State<Arc<JobManager>>,
    Path((job_id, edge_type)): Path<(String, String)>,
    request: Request,
) -> AppResult<Response> {
    tracing::debug!("Serving edge CSV for job: {}, type: {}", job_id, edge_type);

    // Get the job
    let job = job_manager.get_job(&job_id).await?;

    // Validate authentication
    validate_auth(&job, &request)?;

    // Find the edge mapping
    let edge_mapping = job
        .mapping
        .edges
        .iter()
        .find(|e| e.edge_type == edge_type)
        .ok_or_else(|| AppError::NotFound(format!("Edge type not found: {}", edge_type)))?;

    // Generate CSV content
    let csv_content = generate_edge_csv(edge_mapping)?;

    // Return response with proper headers
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/csv; charset=utf-8")],
        csv_content,
    )
        .into_response())
}

/// Generate CSV content for nodes
fn generate_node_csv(node_mapping: &NodeMapping) -> AppResult<String> {
    let mut csv_content = String::new();

    // Generate header
    csv_content.push_str(&DataTransformer::generate_csv_header(&node_mapping.columns));
    csv_content.push('\n');

    // Generate data rows
    for row_data in &node_mapping.data {
        csv_content.push_str(&DataTransformer::generate_csv_row(row_data, &node_mapping.columns));
        csv_content.push('\n');
    }

    Ok(csv_content)
}

/// Generate CSV content for edges
fn generate_edge_csv(edge_mapping: &EdgeMapping) -> AppResult<String> {
    let mut csv_content = String::new();

    // Generate header
    csv_content.push_str(&DataTransformer::generate_csv_header(&edge_mapping.columns));
    csv_content.push('\n');

    // Generate data rows
    for row_data in &edge_mapping.data {
        csv_content.push_str(&DataTransformer::generate_csv_row(row_data, &edge_mapping.columns));
        csv_content.push('\n');
    }

    Ok(csv_content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::models::{ColumnMapping, NodeMapping};
    use std::collections::HashMap;

    #[test]
    fn test_csv_field_escaping() {
        assert_eq!(DataTransformer::escape_csv_field("simple"), "simple");
        assert_eq!(DataTransformer::escape_csv_field("with,comma"), "\"with,comma\"");
        assert_eq!(DataTransformer::escape_csv_field("with\"quote"), "\"with\"\"quote\"");
    }

    #[test]
    fn test_type_conversion() {
        assert_eq!(DataTransformer::convert_type("123", "integer"), "123");
        assert_eq!(DataTransformer::convert_type("123.45", "float"), "123.45");
        assert_eq!(DataTransformer::convert_type("true", "boolean"), "true");
        assert_eq!(DataTransformer::convert_type("text", "string"), "text");
    }

    #[test]
    fn test_generate_csv_header() {
        let columns = vec![
            ColumnMapping {
                source_column: "id".to_string(),
                target_property: "id".to_string(),
                data_type: None,
                trim: true,
            },
            ColumnMapping {
                source_column: "name".to_string(),
                target_property: "name".to_string(),
                data_type: None,
                trim: true,
            },
        ];

        let header = DataTransformer::generate_csv_header(&columns);
        assert_eq!(header, "id,name");
    }

    #[test]
    fn test_generate_csv_row() {
        let mut data = HashMap::new();
        data.insert("id".to_string(), serde_json::json!("1"));
        data.insert("name".to_string(), serde_json::json!("John Doe"));

        let columns = vec![
            ColumnMapping {
                source_column: "id".to_string(),
                target_property: "id".to_string(),
                data_type: None,
                trim: true,
            },
            ColumnMapping {
                source_column: "name".to_string(),
                target_property: "name".to_string(),
                data_type: None,
                trim: true,
            },
        ];

        let row = DataTransformer::generate_csv_row(&data, &columns);
        assert_eq!(row, "1,John Doe");
    }

    #[test]
    fn test_generate_node_csv() {
        let mut data = HashMap::new();
        data.insert("id".to_string(), serde_json::json!("1"));
        data.insert("name".to_string(), serde_json::json!("Alice"));

        let node_mapping = NodeMapping {
            label: "Person".to_string(),
            columns: vec![
                ColumnMapping {
                    source_column: "id".to_string(),
                    target_property: "id".to_string(),
                    data_type: None,
                    trim: true,
                },
                ColumnMapping {
                    source_column: "name".to_string(),
                    target_property: "name".to_string(),
                    data_type: None,
                    trim: true,
                },
            ],
            data: vec![data],
        };

        let csv = generate_node_csv(&node_mapping).unwrap();
        assert!(csv.contains("id,name"));
        assert!(csv.contains("1,Alice"));
    }
}
