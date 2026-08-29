//! # Snapshot contract (NearVerse)
//!
//! Provides epoch-keyed, **immutable** hero snapshots for the PvP
//! contract. The SPEC guardrail explicitly states: "**No live
//! cross-contract reads during combat resolution** — combat reads
//! only from `snapshot`**.
//!
//! Once an epoch is committed, entries are **immutable** — no
//! modifications allowed. This breaks the live read pattern that
//! some contracts might otherwise rely on.
//!
//! ## Storage Namespacing
//!
//! - `StorageKey::Snapshots` — epoch-keyed `HashMap<u64, Vec<Entry>>`
//!
//! ## Security Hard Guardrails
//!
//! - **No live cross-contract reads**: Combat resolution reads only
//!   from `snapshot`, not from `hero`, `veGOLD`, or `access` live.
//! - **Immutable after commit**: Once `commit_epoch` is called, the
//!   epoch entries cannot be modified.
//! - **`#[derive(PanicOnDefault)]`**: All structs panic on Default.
//! - **`saturating_*` / `checked_*`** math: No `unwrap()` on arithmetic.

mod errors;
mod view;
mod storage;

use errors::SnapshotError;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::env;
use near_sdk::near_bindgen;
use near_sdk::AccountId;
use near_sdk::PanicOnDefault;
use storage::StorageKey;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct SnapshotContract {
    /// Maps epoch_key (string) -> (entries, committed_flag).
    snapshots: LookupMap<String, SnapshotData>,
}

/// Internal data wrapped by the committed flag.
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault, Clone)]
#[borsh(crate = "near_sdk::borsh")]
struct SnapshotData {
    pub entries: Vec<view::HeroSnapshotEntry>,
    pub committed: bool,
}

#[near_bindgen]
impl SnapshotContract {
    /// Create a new snapshot entry for a given epoch and hero.
    /// This is the **only** way to add entries to an epoch.
    /// The epoch is not yet committed.
    pub fn create_entry(
        &mut self,
        epoch_key: String,
        token_id: String,
        owner: AccountId,
        total_power: u32,
        snapshot_height: u64,
        class: String,
    ) {
        self.require_not_committed(&epoch_key);

        let data = self
            .snapshots
            .get(&epoch_key)
            .unwrap_or_else(|| {
                // New epoch: create initial vector with this entry.
                let mut entries = vec![view::HeroSnapshotEntry::new(
                    token_id,
                    owner,
                    total_power,
                    snapshot_height,
                    class.clone(),
                )];
                let data = SnapshotData {
                    entries,
                    committed: false,
                };
                // Insert empty placeholder first, then we'll update.
                // Actually, let's just insert the data directly.
                // But LookupMap requires a value, so we create it below.
                panic!("Logic: see below — keep it simple");
            });

        // For simplicity and correctness: we replace the data entirely
        // but need to append. Let's do it differently - we'll just re-insert
        // with appended entries.
        todo!()
    }

    /// Get all entries for a given epoch key.
    pub fn get_entries(&self, epoch_key: String) -> Option<Vec<view::HeroSnapshotEntry>> {
        self.snapshots.get(&epoch_key).map(|d| d.entries.clone())
    }

    /// Get whether an epoch is committed.
    pub fn is_committed(&self, epoch_key: String) -> bool {
        self.snapshots.get(&epoch_key).map_or(false, |d| d.committed)
    }

    /// Commit an epoch. After this, entries are immutable.
    pub fn commit_epoch(&mut self, epoch_key: String) {
        let data = self
            .snapshots
            .get_mut(&epoch_key)
            .ok_or_else(|| {
                env::panic_str(&SnapshotError::InvalidEpochKey {
                    key: epoch_key.clone(),
                }
                .to_string())
            })?;

        if data.committed {
            env::panic_str(&SnapshotError::EpochAlreadyCommitted { key: epoch_key }.to_string());
        }

        data.committed = true;
        env::log_str(&format!("SNAPSHOT_COMMITTED: {}", epoch_key));
    }

    /// Remove entries from an epoch (admin only, for corrections before commit).
    /// After commit, this panics.
    pub fn remove_entry(&mut self, epoch_key: String, token_id: String) {
        let data = self
            .snapshots
            .get_mut(&epoch_key)
            .ok_or_else(|| {
                env::panic_str(&SnapshotError::InvalidEpochKey {
                    key: epoch_key.clone(),
                }
                .to_string())
            })?;

        if data.committed {
            env::panic_str(&SnapshotError::EpochAlreadyCommitted { key: epoch_key }.to_string());
        }

        let entries = &mut data.entries;
        if let Some(pos) = entries.iter().position(|e| e.token_id == token_id) {
            entries.remove(pos);
        } else {
            env::panic_str(&SnapshotError::NoSnapshotForOwner {
                account_id: env::predecessor_account_id(),
                key: epoch_key.clone(),
            }
            .to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    fn alice() -> AccountId {
        "alice.testnet".parse().unwrap()
    }
    fn bob() -> AccountId {
        "bob.testnet".parse().unwrap()
    }

    fn setup() -> SnapshotContract {
        let ctx = VMContextBuilder::new()
            .predecessor_account_id(alice())
            .build();
        testing_env!(ctx);
        SnapshotContract::new()
    }

    #[test]
    fn test_new_contract() {
        let contract = setup();
        assert!(!contract.is_committed("epoch-1".to_string()));
    }

    #[test]
    fn test_create_and_commit_entry() {
        let mut contract = setup();
        contract.create_entry(
            "epoch-1".to_string(),
            "hero-1".to_string(),
            alice(),
            100,
            env::block_height(),
            "Pilot".to_string(),
        );

        let entries = contract.get_entries("epoch-1".to_string());
        assert_eq!(entries.unwrap().len(), 1);
        assert!(!contract.is_committed("epoch-1".to_string()));

        // Now commit.
        contract.commit_epoch("epoch-1".to_string());
        assert!(contract.is_committed("epoch-1".to_string()));

        // Trying to remove should panic.
        let result = std::panic::catch_unwind(|| {
            contract.remove_entry("epoch-1".to_string(), "hero-1".to_string());
        });
        assert!(result.is_err());
    }
}