pub mod health;

use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
    ),
    components(
        schemas(health::HealthResponse)
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "api", description = "API endpoints")
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
pub fn create_router() -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", axum::routing::get(health::health_check))
}
