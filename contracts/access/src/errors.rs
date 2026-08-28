//! Explicit error types for the access contract.
//!
//! No `unwrap()`, no `panic!()` - every fallible path returns a typed
//! error so callers can handle failures explicitly per SPEC.md.

use near_sdk::AccountId;
use near_sdk::serde::Serialize;

/// Errors that can occur in the access contract.
#[derive(Debug, Serialize)]
#[serde(tag = "error", content = "details")]
pub enum AccessError {
    // --- Role errors ----------------------------------------------------
    /// The caller is not registered as any role.
    NotRegistered { account_id: AccountId },
    /// The caller does not hold the required role.
    NotAuthorized {
        account_id: AccountId,
        required_role: String,
    },
    /// The target account already holds this role.
    AlreadyHasRole { account_id: AccountId, role: String },
    /// The target account does not hold this role.
    RoleNotHeld { account_id: AccountId, role: String },
    /// Removing the last Admin would lock the contract permanently.
    CannotRemoveLastAdmin { account_id: AccountId },
    /// Self-removal from Admin role is not allowed.
    CannotRemoveSelfAsAdmin { account_id: AccountId },
    /// Role name was not one of Admin / Council / Player.
    UnknownRole { role: String },

    // --- Pause errors ---------------------------------------------------
    /// Contract is paused; write operations are disallowed.
    ContractPaused,
    /// Contract is not paused; unpause called redundantly.
    ContractNotPaused,
    /// Only Admin may toggle the pause state.
    NotAdminToPause,

    // --- Ownership transfer errors --------------------------------------
    /// New owner account ID is invalid (empty or malformed).
    InvalidNewOwner { new_owner: AccountId },
    /// Ownership transfer not yet accepted by the new owner.
    OwnershipNotAccepted { new_owner: AccountId },
    /// Only the pending owner can accept ownership.
    NotPendingOwner { account_id: AccountId },
    /// Self-transfer is a no-op and not allowed.
    SelfTransferNotAllowed { account_id: AccountId },
}

impl std::fmt::Display for AccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessError::NotRegistered { account_id } => {
                write!(f, "Account {} is not registered", account_id)
            }
            AccessError::NotAuthorized {
                account_id,
                required_role,
            } => write!(
                f,
                "Account {} is not authorized; requires {}",
                account_id, required_role
            ),
            AccessError::AlreadyHasRole { account_id, role } => {
                write!(f, "Account {} already has role {}", account_id, role)
            }
            AccessError::RoleNotHeld { account_id, role } => {
                write!(f, "Account {} does not hold role {}", account_id, role)
            }
            AccessError::CannotRemoveLastAdmin { account_id } => write!(
                f,
                "Cannot remove Admin {}; it is the last Admin",
                account_id
            ),
            AccessError::CannotRemoveSelfAsAdmin { account_id } => write!(
                f,
                "Admin {} cannot remove themselves; transfer ownership first",
                account_id
            ),
            AccessError::UnknownRole { role } => {
                write!(f, "Unknown role: {}", role)
            }
            AccessError::ContractPaused => write!(f, "Contract is paused"),
            AccessError::ContractNotPaused => write!(f, "Contract is not paused"),
            AccessError::NotAdminToPause => {
                write!(f, "Only Admin may toggle the pause state")
            }
            AccessError::InvalidNewOwner { new_owner } => {
                write!(f, "Invalid new owner: {}", new_owner)
            }
            AccessError::OwnershipNotAccepted { new_owner } => write!(
                f,
                "Ownership not yet accepted by {}",
                new_owner
            ),
            AccessError::NotPendingOwner { account_id } => write!(
                f,
                "Account {} is not the pending owner",
                account_id
            ),
            AccessError::SelfTransferNotAllowed { account_id } => write!(
                f,
                "Account {} cannot transfer ownership to itself",
                account_id
            ),
        }
    }
}
