mod api;
mod config;
mod error;
mod shutdown;

use axum::Router;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::AppConfig;
use error::AppResult;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Load configuration
    let config = AppConfig::load()?;

    // Initialize tracing with configured log level
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.logging.level.clone().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!(
        "Starting FalkorDB Importer Backend v{}",
        env!("CARGO_PKG_VERSION")
    );
    tracing::info!("Configuration loaded: {:?}", config);

    // Set up static file serving with SPA fallback
    tracing::info!("Serving frontend from: {}", config.server.frontend_dir);

    let index_path = format!("{}/index.html", config.server.frontend_dir);
    let serve_dir =
        ServeDir::new(&config.server.frontend_dir).not_found_service(ServeFile::new(&index_path));

    // Build API router
    let api_router = api::create_router();

    // Build main application router
    let app = Router::new()
        .nest("/api", api_router)
        .fallback_service(serve_dir)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    // Start server with graceful shutdown
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("Listening on http://{}", addr);
    tracing::info!(
        "API documentation available at http://{}/api/swagger-ui/",
        addr
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown::shutdown_signal())
        .await?;

    tracing::info!("Server shutdown complete");

    Ok(())
}
