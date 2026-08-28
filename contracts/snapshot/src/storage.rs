//! Borsh-serializable storage key namespace for the snapshot contract.

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};

/// All storage keys used by the snapshot contract.
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    /// Maps `epoch_key` -> `Vec<HeroSnapshotEntry>`.
    Snapshots,
}

impl StorageKey {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec()
            .expect("StorageKey serialization must not fail")
    }
}
