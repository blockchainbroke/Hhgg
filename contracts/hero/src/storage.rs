//! Borsh-serializable storage key namespace for the hero contract.

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};

/// All storage keys used by the hero contract.
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    /// Maps `token_id` -> `Hero`.
    Heroes,
    /// Maps `AccountId` -> `UnorderedSet<TokenId>` (owner enumeration).
    OwnerTokens,
    /// Maps `token_id` -> `AccountId` (reverse, for transfers).
    TokenOwners,
    /// Maps `token_id` -> `UnorderedSet<AccountId>` (NEP-178 approvals).
    Approvals,
}

impl StorageKey {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec()
            .expect("StorageKey serialization must not fail")
    }
}
