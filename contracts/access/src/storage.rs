//! Borsh-serializable storage key namespace for the access contract.
//!
//! Per SPEC.md §Storage Namespacing, every contract uses its own
//! `StorageKey` variant so keys never collide when multiple NearVerse
//! contracts share a single account's storage.

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};

/// All storage keys used by the access contract.
/// No key may be duplicated across contracts in the workspace.
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    /// Maps `AccountId` -> `RoleEntry`. The role registry.
    Roles,
    /// Stores the pending owner during the two-step transfer flow.
    PendingOwner,
}

impl StorageKey {
    /// Serialize this key to a byte vector for use as a `LookupMap` key.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec()
            .expect("StorageKey serialization must not fail")
    }
}
