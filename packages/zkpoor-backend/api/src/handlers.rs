use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

use crate::types::*;
use zkpoor_core::proof::{ProofGenerator, ProofVerifier};

#[derive(Debug, Deserialize)]
pub struct VerifyQuery {
    pub proof: String,
    pub public_inputs: String,
}

/// POST /prove - Generate a proof for the given UTXOs
pub async fn prove_handler(
    Json(request): Json<ProveRequest>,
) -> Result<Json<ProveResponse>, (StatusCode, Json<ApiError>)> {
    tracing::info!("Received proof generation request with {} UTXOs", request.utxos.len());
    
    // Validate input
    if request.utxos.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "No UTXOs provided".to_string(),
                details: None,
            }),
        ));
    }

    if request.utxos.len() != request.ownership_proofs.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "Mismatch between UTXOs and ownership proofs".to_string(),
                details: Some("Each UTXO must have a corresponding ownership proof".to_string()),
            }),
        ));
    }

    // Generate proof using core library (mocked for now)
    let proof_generator = ProofGenerator::new();
    match proof_generator.generate_proof_async(&request.utxos, &request.ownership_proofs).await {
        Ok(proof_id) => {
            tracing::info!("Proof generation started with ID: {}", proof_id);
            Ok(Json(ProveResponse {
                proof_id,
                status: ProofStatus::Pending,
                message: "Proof generation initiated successfully".to_string(),
            }))
        }
        Err(e) => {
            tracing::error!("Failed to generate proof: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "Failed to initiate proof generation".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// GET /proof/{proof_id} - Get proof data by ID
pub async fn get_proof_handler(
    Path(proof_id): Path<Uuid>,
) -> Result<Json<ProofData>, (StatusCode, Json<ApiError>)> {
    tracing::info!("Fetching proof data for ID: {}", proof_id);
    
    let proof_generator = ProofGenerator::new();
    match proof_generator.get_proof_status(proof_id).await {
        Ok(Some(proof_data)) => {
            tracing::info!("Found proof data for ID: {} with status: {:?}", proof_id, proof_data.status);
            Ok(Json(proof_data))
        }
        Ok(None) => {
            tracing::warn!("Proof not found for ID: {}", proof_id);
            Err((
                StatusCode::NOT_FOUND,
                Json(ApiError {
                    error: "Proof not found".to_string(),
                    details: Some(format!("No proof found with ID: {}", proof_id)),
                }),
            ))
        }
        Err(e) => {
            tracing::error!("Error fetching proof {}: {}", proof_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "Failed to fetch proof data".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// POST /proof-verify - Verify a proof
pub async fn verify_proof_handler(
    Query(params): Query<VerifyQuery>,
) -> Result<Json<VerifyResponse>, (StatusCode, Json<ApiError>)> {
    tracing::info!("Received proof verification request");
    
    // Validate query parameters
    if params.proof.is_empty() || params.public_inputs.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "Missing required parameters".to_string(),
                details: Some("Both 'proof' and 'public_inputs' parameters are required".to_string()),
            }),
        ));
    }

    // Verify proof using core library (mocked for now)
    let proof_verifier = ProofVerifier::new();
    match proof_verifier.verify_proof(&params.proof, &params.public_inputs).await {
        Ok((is_valid, total_amount)) => {
            let message = if is_valid {
                "Proof verification successful".to_string()
            } else {
                "Proof verification failed".to_string()
            };
            
            tracing::info!("Proof verification result: valid={}, amount={:?}", is_valid, total_amount);
            
            Ok(Json(VerifyResponse {
                is_valid,
                total_amount,
                verified_at: Utc::now(),
                message,
            }))
        }
        Err(e) => {
            tracing::error!("Error during proof verification: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "Proof verification failed".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// GET /health - Health check endpoint
pub async fn health_handler() -> Json<HashMap<String, String>> {
    let mut response = HashMap::new();
    response.insert("status".to_string(), "healthy".to_string());
    response.insert("timestamp".to_string(), Utc::now().to_rfc3339());
    response.insert("service".to_string(), "zkpoor-api".to_string());
    Json(response)
}