//! Explicit error types for the access contract.
//!
//! No `unwrap()`, no `panic!()` — every fallible path returns a typed
//! error so callers can handle failures explicitly per SPEC.md.

use near_sdk::AccountId;
use near_sdk::json_types::U128;
use near_sdk::serde::Serialize;

/// Errors that can occur in this contract.
/// All variants include context so error messages are self-describing.
#[derive(Debug, Serialize)]
#[serde(tag = "error", content = "details")]
pub enum AccessError {
    // ── Role errors ─────────────────────────────────────────────────────────
    /// The caller is not registered as any role.
    NotRegistered { account_id: AccountId },
    /// The caller does not hold the required role.
    NotAuthorized { account_id: AccountId, required_role: String },
    /// The target account already holds this role.
    AlreadyHasRole { account_id: AccountId, role: String },
    /// The target account does not hold this role.
    RoleNotHeld { account_id: AccountId, role: String },
    /// Removing the last Admin would lock the contract permanently.
    CannotRemoveLastAdmin { account_id: AccountId },
    /// Self-removal from Admin role is not allowed (prevents lock-out).
    CannotRemoveSelfAsAdmin { account_id: AccountId },

    // ── Pause errors ────────────────────────────────────────────────────────
    /// Contract is paused; write operations are disallowed.
    ContractPaused,
    /// Contract is not paused; unpause called redundantly.
    ContractNotPaused,
    /// Only Admin may toggle the pause state.
    NotAdminToPause,

    // ── Ownership transfer errors ───────────────────────────────────────────
    /// New owner account ID is invalid (empty or malformed).
    InvalidNewOwner { new_owner: AccountId },
    /// Ownership transfer not yet accepted by the new owner.
    OwnershipNotAccepted { new_owner: AccountId },
    /// Only the pending owner can accept ownership.
    NotPendingOwner { account_id: AccountId },
    /// Self-transfer is a no-op and disallowed.
    SelfTransfer { account_id: AccountId },

    // ── Storage / registration errors ───────────────────────────────────────
    /// Registration requires a deposit.
    RegistrationRequiresDeposit { min_deposit: U128 },
    /// Storage write failed (NEAR SDK boundary error).
    StorageWriteFailed,
}

/// Helper to create a "not authorized" error with the caller's account.
pub fn not_authorized(account_id: &AccountId, required_role: &str) -> AccessError {
    AccessError::NotAuthorized {
        account_id: account_id.clone(),
        required_role: required_role.to_string(),
    }
}

/// Helper to create a "not registered" error.
pub fn not_registered(account_id: &AccountId) -> AccessError {
    AccessError::NotRegistered {
        account_id: account_id.clone(),
    }
}
