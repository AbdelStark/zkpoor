use axum::Router;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::routes::create_routes;

pub fn create_app() -> Router {
    create_routes()
        .layer(TraceLayer::new_for_http())
}

pub async fn run_server(port: u16) -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "zkpoor_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = create_app();

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting zkpoor API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}