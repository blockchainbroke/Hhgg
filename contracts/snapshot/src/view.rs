//! Domain types for the snapshot contract.

use near_sdk::AccountId;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::Serialize;

/// An immutable snapshot of a hero's stats at the time an epoch was committed.
/// Once an epoch is committed, no entry can be modified — this is the
/// foundation of the SPEC guardrail that prohibits live cross-contract reads
/// in the PvP contract.
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct HeroSnapshotEntry {
    pub token_id: String,
    pub owner: AccountId,
    pub total_power: u32,
    /// Block height at which this snapshot was taken.
    pub snapshot_height: u64,
    /// Class as a string (avoids needing to import hero types here).
    pub class: String,
}

impl HeroSnapshotEntry {
    pub fn new(
        token_id: String,
        owner: AccountId,
        total_power: u32,
        snapshot_height: u64,
        class: String,
    ) -> Self {
        Self {
            token_id,
            owner,
            total_power,
            snapshot_height,
            class,
        }
    }
}

/// An immutable snapshot of all heroes in a given epoch.
/// The `committed` flag indicates whether the epoch is sealed and immutable.
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct HeroSnapshotView {
    /// Human-readable epoch key (e.g. "epoch-2026-08").
    pub epoch_key: String,
    /// All hero entries in this epoch.
    pub entries: Vec<HeroSnapshotEntry>,
    /// Block height at which the epoch was committed. `0` if not yet committed.
    pub committed_height: u64,
    /// `true` if this epoch has been sealed and is now immutable.
    pub committed: bool,
}

impl HeroSnapshotView {
    /// Returns `true` if this snapshot contains the given owner's heroes.
    pub fn has_owner(&self, account_id: &AccountId) -> bool {
        self.entries.iter().any(|e| &e.owner == account_id)
    }

    /// Returns the total count of heroes in this epoch.
    pub fn hero_count(&self) -> usize {
        self.entries.len()
    }
}
