use axum::{routing::get, Router};
use std::env;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting FalkorDB Importer Backend");

    // Set up static file serving with SPA fallback
    let frontend_dir = env::var("FRONTEND_DIR").unwrap_or_else(|_| "../frontend/dist".to_string());
    tracing::info!("Serving frontend from: {}", frontend_dir);
    
    let index_path = format!("{}/index.html", frontend_dir);
    let serve_dir = ServeDir::new(&frontend_dir)
        .not_found_service(ServeFile::new(&index_path));

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        // Future API routes should be nested under /api
        .fallback_service(serve_dir)
        .layer(CorsLayer::permissive());

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        assert_eq!(response, "OK");
    }
}
