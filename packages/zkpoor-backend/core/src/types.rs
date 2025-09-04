use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Utxo {
    pub txid: String,
    pub vout: u32,
    pub amount: u64, // in satoshis
    pub script_pubkey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipProof {
    pub signature: String,
    pub challenge: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProofStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProofData {
    pub proof_id: Uuid,
    pub status: ProofStatus,
    pub total_amount: Option<u64>, // Total BTC in satoshis
    pub proof: Option<String>, // Base64 encoded proof
    pub public_inputs: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct ZkProof {
    pub proof_data: Vec<u8>,
    pub public_inputs: serde_json::Value,
    pub total_amount: u64,
}

impl ZkProof {
    pub fn to_base64(&self) -> String {
        base64::encode(&self.proof_data)
    }
}

// Mock base64 and hex utilities
mod base64 {
    use crate::types::hex;

    pub fn encode(data: &[u8]) -> String {
        // Mock base64 encoding for now
        format!("mock_proof_{}", hex::encode(data))
    }
}

mod hex {
    pub fn encode(data: &[u8]) -> String {
        data.iter().map(|b| format!("{:02x}", b)).collect()
    }
}