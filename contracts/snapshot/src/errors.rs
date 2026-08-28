//! Error types for the snapshot contract.

use near_sdk::AccountId;
use near_sdk::serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "error", content = "details")]
pub enum SnapshotError {
    /// The epoch key was empty or malformed.
    InvalidEpochKey { key: String },
    /// The requested epoch was not found.
    EpochNotFound { key: String },
    /// The owner has no hero snapshots in the requested epoch.
    NoSnapshotForOwner { account_id: AccountId, key: String },
    /// The epoch has already been committed; no further changes allowed.
    EpochAlreadyCommitted { key: String },
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapshotError::InvalidEpochKey { key } => {
                write!(f, "Invalid epoch key: {}", key)
            }
            SnapshotError::EpochNotFound { key } => {
                write!(f, "Snapshot epoch not found: {}", key)
            }
            SnapshotError::NoSnapshotForOwner { account_id, key } => {
                write!(
                    f,
                    "No snapshot for {} in epoch {}",
                    account_id, key
                )
            }
            SnapshotError::EpochAlreadyCommitted { key } => {
                write!(f, "Epoch {} is already committed; immutable", key)
            }
        }
    }
}
