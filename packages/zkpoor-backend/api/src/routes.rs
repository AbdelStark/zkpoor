use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{CorsLayer, Any};

use crate::handlers::{prove_handler, get_proof_handler, verify_proof_handler, health_handler};

pub fn create_routes() -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_handler))
        
        // Proof generation endpoint
        .route("/prove", post(prove_handler))
        
        // Get proof by ID
        .route("/proof/:proof_id", get(get_proof_handler))
        
        // Verify proof
        .route("/proof-verify", post(verify_proof_handler))
        
        // CORS layer to allow frontend to call API
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
}