//! Error types for the veGOLD vault contract.

use near_sdk::AccountId;
use near_sdk::serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "error", content = "details")]
pub enum VeGOLDError {
    /// Account does not hold the hero NFT.
    NotHeroOwner { account_id: AccountId, token_id: String },
    /// Insufficient veGOLD balance.
    InsufficientBalance { account_id: AccountId, required: u128, available: u128 },
    /// Lock duration is below the minimum.
    LockTooShort { duration_secs: u64, min: u64 },
    /// Lock duration exceeds the maximum.
    LockTooLong { duration_secs: u64, max: u64 },
    /// Contract is paused.
    ContractPaused,
    /// Lock not found.
    LockNotFound { account_id: AccountId },
    /// Invalid lock ID (negative or overflow).
    InvalidLockId { id: u64 },
    /// Max boost BPS exceeds the protocol limit.
    BoostExceedsMax { requested: u32, max: u32 },
}

impl std::fmt::Display for VeGOLDError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VeGOLDError::NotHeroOwner { account_id, token_id } => {
                write!(f, "Account {} is not the owner of hero {}", account_id, token_id)
            }
            VeGOLDError::InsufficientBalance { account_id, required, available } => {
                write!(f, "Account {} has {} veGOLD but needs {}", account_id, available, required)
            }
            VeGOLDError::LockTooShort { duration_secs, min } => {
                write!(f, "Lock duration {} is less than minimum {}", duration_secs, min)
            }
            VeGOLDError::LockTooLong { duration_secs, max } => {
                write!(f, "Lock duration {} exceeds maximum {}", duration_secs, max)
            }
            VeGOLDError::ContractPaused => write!(f, "veGOLD contract is paused"),
            VeGOLDError::LockNotFound { account_id } => {
                write!(f, "No lock found for account {}", account_id)
            }
            VeGOLDError::InvalidLockId { id } => {
                write!(f, "Invalid lock ID: {}", id)
            }
            VeGOLDError::BoostExceedsMax { requested, max } => {
                write!(f, "Requested boost {} bps exceeds max {} bps", requested, max)
            }
        }
    }
}
