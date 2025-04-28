use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Represents the actual data being transferred.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransferData {
    pub transaction_id: String,
    pub payload: Vec<u8>, // Actual bytes of data
    pub created_at: DateTime<Utc>,
}

/// Wrapper structure sent over the network.
/// Contains the nonce needed for key derivation and the encrypted payload.
#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptedPacket {
    pub nonce: Vec<u8>,
    pub encrypted_payload: Vec<u8>,
}
