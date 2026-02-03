use crate::error::{AppError, AppResult};
use crate::jobs::models::{Job, JobId, MappingConfig};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

/// Job manager for creating, tracking, and cleaning up jobs
#[derive(Clone)]
pub struct JobManager {
    jobs: Arc<RwLock<HashMap<JobId, Job>>>,
    default_ttl_seconds: u64,
}

impl JobManager {
    /// Create a new job manager with a default TTL
    pub fn new(default_ttl_seconds: u64) -> Self {
        let manager = Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            default_ttl_seconds,
        };

        // Start background cleanup task
        let cleanup_jobs = manager.jobs.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Check every minute
            loop {
                interval.tick().await;
                // Cleanup task with error handling
                if let Err(e) = Self::cleanup_expired_jobs_with_error_handling(&cleanup_jobs).await
                {
                    tracing::error!("Cleanup task encountered an error: {}", e);
                }
            }
        });

        manager
    }

    /// Create a new job
    pub async fn create_job(
        &self,
        mapping: MappingConfig,
        auth_token: Option<String>,
    ) -> AppResult<Job> {
        let job_id = uuid::Uuid::new_v4().to_string();
        let job = Job::new(job_id, mapping, self.default_ttl_seconds, auth_token);

        let mut jobs = self.jobs.write().await;
        jobs.insert(job.id.clone(), job.clone());

        tracing::info!("Created job: {}", job.id);
        Ok(job)
    }

    /// Get a job by ID
    pub async fn get_job(&self, job_id: &str) -> AppResult<Job> {
        let jobs = self.jobs.read().await;
        let job = jobs
            .get(job_id)
            .ok_or_else(|| AppError::NotFound(format!("Job not found: {}", job_id)))?;

        if job.is_expired() {
            return Err(AppError::NotFound(format!("Job has expired: {}", job_id)));
        }

        Ok(job.clone())
    }

    /// Update job status
    #[allow(dead_code)]
    pub async fn update_job(&self, job: Job) -> AppResult<()> {
        let mut jobs = self.jobs.write().await;
        jobs.insert(job.id.clone(), job);
        Ok(())
    }

    /// Delete a job
    pub async fn delete_job(&self, job_id: &str) -> AppResult<()> {
        let mut jobs = self.jobs.write().await;
        jobs.remove(job_id)
            .ok_or_else(|| AppError::NotFound(format!("Job not found: {}", job_id)))?;

        tracing::info!("Deleted job: {}", job_id);
        Ok(())
    }

    /// List all active jobs
    pub async fn list_jobs(&self) -> Vec<Job> {
        let jobs = self.jobs.read().await;
        jobs.values()
            .filter(|job| !job.is_expired())
            .cloned()
            .collect()
    }

    /// Clean up expired jobs (called periodically) with error handling
    async fn cleanup_expired_jobs_with_error_handling(
        jobs: &Arc<RwLock<HashMap<JobId, Job>>>,
    ) -> Result<(), String> {
        let mut jobs_guard = jobs.write().await;
        let expired_jobs: Vec<JobId> = jobs_guard
            .iter()
            .filter(|(_, job)| job.is_expired())
            .map(|(id, _)| id.clone())
            .collect();

        for job_id in expired_jobs {
            jobs_guard.remove(&job_id);
            tracing::info!("Cleaned up expired job: {}", job_id);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::models::MappingConfig;

    #[tokio::test]
    async fn test_create_job() {
        let manager = JobManager::new(3600);
        let mapping = MappingConfig {
            nodes: vec![],
            edges: vec![],
        };

        let job = manager.create_job(mapping, None).await.unwrap();
        assert!(!job.id.is_empty());
    }

    #[tokio::test]
    async fn test_get_job() {
        let manager = JobManager::new(3600);
        let mapping = MappingConfig {
            nodes: vec![],
            edges: vec![],
        };

        let created_job = manager.create_job(mapping, None).await.unwrap();
        let retrieved_job = manager.get_job(&created_job.id).await.unwrap();

        assert_eq!(created_job.id, retrieved_job.id);
    }

    #[tokio::test]
    async fn test_delete_job() {
        let manager = JobManager::new(3600);
        let mapping = MappingConfig {
            nodes: vec![],
            edges: vec![],
        };

        let job = manager.create_job(mapping, None).await.unwrap();
        manager.delete_job(&job.id).await.unwrap();

        let result = manager.get_job(&job.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_jobs() {
        let manager = JobManager::new(3600);
        let mapping = MappingConfig {
            nodes: vec![],
            edges: vec![],
        };

        manager.create_job(mapping.clone(), None).await.unwrap();
        manager.create_job(mapping, None).await.unwrap();

        let jobs = manager.list_jobs().await;
        assert_eq!(jobs.len(), 2);
    }
}
