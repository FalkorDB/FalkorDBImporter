use crate::error::AppResult;
use crate::jobs::{Job, JobManager, MappingConfig};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

/// Request to create a new job
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateJobRequest {
    /// Mapping configuration for the job
    pub mapping: MappingConfig,
    /// Optional authentication token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
}

/// Response with job information
#[derive(Debug, Serialize, ToSchema)]
pub struct JobResponse {
    pub job: Job,
}

/// Create a new job
#[utoipa::path(
    post,
    path = "/jobs",
    request_body = CreateJobRequest,
    responses(
        (status = 201, description = "Job created successfully", body = JobResponse),
        (status = 400, description = "Invalid request")
    ),
    tag = "jobs"
)]
pub async fn create_job(
    State(job_manager): State<Arc<JobManager>>,
    Json(request): Json<CreateJobRequest>,
) -> AppResult<impl IntoResponse> {
    tracing::info!("Creating new job");

    let job = job_manager
        .create_job(request.mapping, request.auth_token)
        .await?;

    Ok((axum::http::StatusCode::CREATED, Json(JobResponse { job })))
}

/// Get a job by ID
#[utoipa::path(
    get,
    path = "/jobs/{job_id}",
    params(
        ("job_id" = String, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Job found", body = JobResponse),
        (status = 404, description = "Job not found")
    ),
    tag = "jobs"
)]
pub async fn get_job(
    State(job_manager): State<Arc<JobManager>>,
    Path(job_id): Path<String>,
) -> AppResult<impl IntoResponse> {
    tracing::debug!("Getting job: {}", job_id);

    let job = job_manager.get_job(&job_id).await?;

    Ok(Json(JobResponse { job }))
}

/// List all active jobs
#[utoipa::path(
    get,
    path = "/jobs",
    responses(
        (status = 200, description = "List of jobs", body = Vec<Job>)
    ),
    tag = "jobs"
)]
pub async fn list_jobs(State(job_manager): State<Arc<JobManager>>) -> AppResult<impl IntoResponse> {
    tracing::debug!("Listing all jobs");

    let jobs = job_manager.list_jobs().await;

    Ok(Json(jobs))
}

/// Delete a job
#[utoipa::path(
    delete,
    path = "/jobs/{job_id}",
    params(
        ("job_id" = String, Path, description = "Job ID")
    ),
    responses(
        (status = 204, description = "Job deleted successfully"),
        (status = 404, description = "Job not found")
    ),
    tag = "jobs"
)]
pub async fn delete_job(
    State(job_manager): State<Arc<JobManager>>,
    Path(job_id): Path<String>,
) -> AppResult<impl IntoResponse> {
    tracing::info!("Deleting job: {}", job_id);

    job_manager.delete_job(&job_id).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::JobManager;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use tower::ServiceExt;

    async fn create_test_router() -> Router {
        let job_manager = Arc::new(JobManager::new(3600));

        Router::new()
            .route("/jobs", axum::routing::post(create_job).get(list_jobs))
            .route(
                "/jobs/{job_id}",
                axum::routing::get(get_job).delete(delete_job),
            )
            .with_state(job_manager)
    }

    #[tokio::test]
    async fn test_create_job_endpoint() {
        let app = create_test_router().await;

        let request_body = serde_json::json!({
            "mapping": {
                "nodes": [],
                "edges": []
            }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/jobs")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }
}
