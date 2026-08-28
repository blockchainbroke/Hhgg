//! Borsh-serializable storage key namespace.
//!
//! Per SPEC.md §Storage Namespacing, every contract module uses its own
//! `StorageKey` variant so keys never collide when multiple NearVerse
//! contracts share a single account's storage.
//!
//! Currently only `StorageKey::Roles` is defined for this contract;
//! the other variants are stubs reserved for future contracts.

use near_sdk::borsh::{BorshSerialize, BorshDeserialize};
use near_sdk::borsh::maybestd::io::{Result, Error};

/// All storage keys used by the access contract.
/// New variants are added here as the contract grows.
/// No key may be duplicated across contracts in the workspace.
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    /// Maps `AccountId` → `RoleEntry`. Stores the role registry.
    Roles,
}

impl StorageKey {
    /// Serialize this key to a byte vector suitable for use as a
    /// `UnorderedMap` or `LookupMap` key in NEAR storage.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().expect("StorageKey serialization must not fail")
    }
}
