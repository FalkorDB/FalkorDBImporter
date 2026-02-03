pub mod csv;
pub mod health;
pub mod jobs;

use crate::jobs::JobManager;
use axum::Router;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        jobs::create_job,
        jobs::get_job,
        jobs::list_jobs,
        jobs::delete_job,
        csv::handlers::serve_node_csv,
        csv::handlers::serve_edge_csv,
    ),
    components(
        schemas(
            health::HealthResponse,
            jobs::CreateJobRequest,
            jobs::JobResponse,
            crate::jobs::models::Job,
            crate::jobs::models::JobStatus,
            crate::jobs::models::MappingConfig,
            crate::jobs::models::NodeMapping,
            crate::jobs::models::EdgeMapping,
            crate::jobs::models::ColumnMapping,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "jobs", description = "Job management endpoints"),
        (name = "csv", description = "CSV data streaming endpoints"),
    ),
    servers(
        (url = "/api", description = "API server")
    ),
    info(
        title = "FalkorDB Importer API",
        version = "0.1.0",
        description = "API for importing data into FalkorDB from various sources",
        license(
            name = "Apache-2.0",
            url = "https://www.apache.org/licenses/LICENSE-2.0"
        )
    )
)]
pub struct ApiDoc;

/// Create the API router with all routes
pub fn create_router(job_manager: Arc<JobManager>) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", axum::routing::get(health::health_check))
        .route(
            "/jobs",
            axum::routing::post(jobs::create_job).get(jobs::list_jobs),
        )
        .route(
            "/jobs/{job_id}",
            axum::routing::get(jobs::get_job).delete(jobs::delete_job),
        )
        .route(
            "/data/{job_id}/nodes/{label}",
            axum::routing::get(csv::serve_node_csv),
        )
        .route(
            "/data/{job_id}/edges/{edge_type}",
            axum::routing::get(csv::serve_edge_csv),
        )
        .with_state(job_manager)
}
