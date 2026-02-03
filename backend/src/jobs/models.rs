use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use utoipa::ToSchema;

/// Unique identifier for a job
pub type JobId = String;

/// Job status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// Job is created and ready to serve CSV data
    Active,
    /// Job is processing data
    Processing,
    /// Job has completed successfully
    Completed,
    /// Job has failed
    Failed { error: String },
    /// Job has expired and will be cleaned up
    Expired,
}

/// Column mapping for data transformation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ColumnMapping {
    /// Source column name
    pub source_column: String,
    /// Target property name
    pub target_property: String,
    /// Data type for conversion
    #[serde(default)]
    pub data_type: Option<String>,
    /// Whether to trim whitespace
    #[serde(default = "default_true")]
    pub trim: bool,
}

fn default_true() -> bool {
    true
}

/// Node mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NodeMapping {
    /// Node label (e.g., "Person", "Product")
    pub label: String,
    /// Column mappings for this node type
    pub columns: Vec<ColumnMapping>,
    /// Sample data for this node type
    #[serde(default)]
    pub data: Vec<HashMap<String, serde_json::Value>>,
}

/// Edge mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EdgeMapping {
    /// Edge type (e.g., "KNOWS", "PURCHASED")
    pub edge_type: String,
    /// Column mappings for this edge type
    pub columns: Vec<ColumnMapping>,
    /// Sample data for this edge type
    #[serde(default)]
    pub data: Vec<HashMap<String, serde_json::Value>>,
}

/// Complete mapping configuration for a job
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MappingConfig {
    /// Node mappings
    pub nodes: Vec<NodeMapping>,
    /// Edge mappings
    pub edges: Vec<EdgeMapping>,
}

/// A data import job
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Job {
    /// Unique job identifier
    pub id: JobId,
    /// Job status
    pub status: JobStatus,
    /// Mapping configuration
    pub mapping: MappingConfig,
    /// Job creation timestamp (Unix epoch in seconds)
    pub created_at: u64,
    /// Job expiration timestamp (Unix epoch in seconds)
    pub expires_at: u64,
    /// Optional authentication token for this job
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
}

impl Job {
    /// Create a new job with a TTL in seconds
    pub fn new(id: JobId, mapping: MappingConfig, ttl_seconds: u64, auth_token: Option<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            id,
            status: JobStatus::Active,
            mapping,
            created_at: now,
            expires_at: now + ttl_seconds,
            auth_token,
        }
    }

    /// Check if the job has expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now > self.expires_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let mapping = MappingConfig {
            nodes: vec![],
            edges: vec![],
        };
        let job = Job::new("test-job".to_string(), mapping, 3600, None);
        
        assert_eq!(job.id, "test-job");
        assert!(matches!(job.status, JobStatus::Active));
        assert!(!job.is_expired());
    }

    #[test]
    fn test_job_expiration() {
        let mapping = MappingConfig {
            nodes: vec![],
            edges: vec![],
        };
        let mut job = Job::new("test-job".to_string(), mapping, 0, None);
        
        // Set expiration to the past
        job.expires_at = 0;
        assert!(job.is_expired());
    }
}
