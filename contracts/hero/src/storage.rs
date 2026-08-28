//! Borsh-serializable storage key namespace for the hero contract.

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};

/// All storage keys used by the hero contract.
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    /// Maps `token_id` -> `Hero`.
    Heroes,
    /// Maps `AccountId` -> `HashSet<TokenId>` (owner enumeration).
    OwnerTokens,
    /// Maps `token_id` -> `AccountId` (reverse, for transfers).
    TokenOwners,
}

impl StorageKey {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec()
            .expect("StorageKey serialization must not fail")
    }
}
