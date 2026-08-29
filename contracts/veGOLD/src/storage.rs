//! Borsh-serializable storage key namespace for the veGOLD contract.

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};

/// All storage keys used by the veGOLD contract.
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    /// Maps `AccountId` -> `Lock`.
    Locks,
    /// Total locked supply snapshot.
    TotalLocked,
    /// Boost config: hero_class -> boost_bps (basis points).
    BoostConfig,
}

impl StorageKey {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec()
            .expect("StorageKey serialization must not fail")
    }
}
