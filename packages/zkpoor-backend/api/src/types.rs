use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Re-export types from core
pub use zkpoor_core::types::{Utxo, OwnershipProof, ProofStatus, ProofData};

#[derive(Debug, Deserialize)]
pub struct ProveRequest {
    pub utxos: Vec<Utxo>,
    pub ownership_proofs: Vec<OwnershipProof>,
}

#[derive(Debug, Serialize)]
pub struct ProveResponse {
    pub proof_id: Uuid,
    pub status: ProofStatus,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub is_valid: bool,
    pub total_amount: Option<u64>,
    pub verified_at: DateTime<Utc>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub details: Option<String>,
}