# zkpoor-backend

Rust backend workspace for zkpoor (Zero Knowledge Proof Of Outstanding Reserves).

## Architecture

This workspace contains two crates:

- **`zkpoor-api`** - REST API server with endpoints for proof generation and verification
- **`zkpoor-core`** - Core proof generation and verification logic

## API Endpoints

### POST `/prove`
Generate a STARK proof for Bitcoin UTXOs.

**Request Body:**
```json
{
  "utxos": [
    {
      "txid": "string",
      "vout": 0,
      "amount": 100000000,
      "script_pubkey": "string"
    }
  ],
  "ownership_proofs": [
    {
      "signature": "string",
      "challenge": "string"
    }
  ]
}
```

**Response:**
```json
{
  "proof_id": "uuid",
  "status": "Pending",
  "message": "Proof generation initiated successfully"
}
```

### GET `/proof/{proof_id}`
Get proof data by ID.

**Response:**
```json
{
  "proof_id": "uuid",
  "status": "Completed",
  "total_amount": 100000000,
  "proof": "base64_encoded_proof",
  "public_inputs": {...},
  "created_at": "2023-01-01T00:00:00Z",
  "completed_at": "2023-01-01T00:00:05Z"
}
```

### POST `/proof-verify`
Verify a STARK proof.

**Query Parameters:**
- `proof`: Base64 encoded proof
- `public_inputs`: JSON encoded public inputs

**Response:**
```json
{
  "is_valid": true,
  "total_amount": 100000000,
  "verified_at": "2023-01-01T00:00:00Z",
  "message": "Proof verification successful"
}
```

### GET `/health`
Health check endpoint.

## Development

### Prerequisites
- Rust 1.70+
- Cargo

### Build
```bash
cargo build
```

### Run API Server
```bash
cargo run --bin zkpoor-api
```

The server will start on port 8080 by default. Set the `PORT` environment variable to use a different port.

### Run Tests
```bash
cargo test
```

## Current Implementation Status

⚠️ **Phase 1 Implementation** - Currently using mocked proof generation and verification:

- ✅ REST API endpoints
- ✅ Async proof generation workflow
- ✅ In-memory proof storage
- ✅ Mock STARK proof generation
- ✅ Mock proof verification
- 🔄 **Todo:** Real STARK proof integration
- 🔄 **Todo:** Database persistence
- 🔄 **Todo:** Cairo program integration

## Future Enhancements

- Replace mock proof generation with real STARK/Cairo implementation
- Add database persistence (PostgreSQL/SQLite)
- Add proper authentication and authorization
- Add rate limiting and caching
- Add comprehensive error handling and logging
- Add metrics and monitoring endpoints