//! # Snapshot contract (NearVerse)
//!
//! Provides epoch-keyed, **immutable** hero snapshots for the PvP
//! contract. The SPEC guardrail explicitly states: "**No live
//! cross-contract reads during combat resolution** — combat reads
//! only from `snapshot`.
//!
//! Once an epoch is committed, entries are **immutable** — no
//! modifications allowed. This breaks the live read pattern that
//! some contracts might otherwise rely on.
//!
//! ## Storage Namespacing
//!
//! - `StorageKey::Snapshots` — epoch-keyed `LookupMap<String, SnapshotData>`
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
mod storage;
mod view;

use errors::SnapshotError;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::env;
use near_sdk::near_bindgen;
use near_sdk::AccountId;
use near_sdk::PanicOnDefault;
use storage::StorageKey;
use view::{HeroSnapshotEntry, HeroSnapshotView};

/// Internal data stored per epoch key.
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
struct SnapshotData {
    entries: Vec<HeroSnapshotEntry>,
    committed: bool,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct SnapshotContract {
    /// Maps `epoch_key` (String) -> `SnapshotData`.
    snapshots: LookupMap<String, SnapshotData>,
}

impl Default for SnapshotContract {
    fn default() -> Self {
        env::panic_str("SnapshotContract must be initialized with new()");
    }
}

#[near_bindgen]
impl SnapshotContract {
    /// Initialize the snapshot contract.
    pub fn new() -> Self {
        Self {
            snapshots: LookupMap::new(StorageKey::Snapshots),
        }
    }

    /// Add a hero entry to an epoch. Fails if the epoch is already committed.
    ///
    /// Callable by any account — caller is responsible for gating access
    /// in production (e.g., via the `access` contract).
    pub fn add_entry(&mut self, entry: HeroSnapshotEntry, epoch_key: String) {
        self.require_valid_epoch_key(&epoch_key);
        self.require_not_committed(&epoch_key);

        let mut data = self
            .snapshots
            .get(&epoch_key)
            .unwrap_or_else(|| SnapshotData {
                entries: Vec::new(),
                committed: false,
            });

        data.entries.push(entry);
        self.snapshots.insert(&epoch_key, &data);
    }

    /// Commit an epoch, sealing all its entries and making them immutable.
    /// After this call, `add_entry` will fail for this epoch.
    pub fn commit_epoch(&mut self, epoch_key: String) {
        self.require_valid_epoch_key(&epoch_key);

        let mut data = self
            .snapshots
            .get(&epoch_key)
            .unwrap_or_else(|| SnapshotData {
                entries: Vec::new(),
                committed: false,
            });

        data.committed = true;
        self.snapshots.insert(&epoch_key, &data);
    }

    /// Return the full snapshot view for an epoch.
    /// Returns `None` if the epoch does not exist.
    pub fn get_snapshot(&self, epoch_key: String) -> Option<HeroSnapshotView> {
        self.snapshots.get(&epoch_key).map(|data| HeroSnapshotView {
            epoch_key: epoch_key.clone(),
            entries: data.entries,
            committed_height: if data.committed {
                env::block_height()
            } else {
                0
            },
            committed: data.committed,
        })
    }

    /// Return only the snapshot entries belonging to `owner` in a given epoch.
    /// Returns `Err` if the epoch doesn't exist or the owner has no entries.
    pub fn get_owner_snapshot(
        &self,
        owner: AccountId,
        epoch_key: String,
    ) -> Result<Vec<HeroSnapshotEntry>, SnapshotError> {
        let data = self.snapshots.get(&epoch_key).ok_or_else(|| {
            SnapshotError::EpochNotFound {
                key: epoch_key.clone(),
            }
        })?;

        let entries: Vec<HeroSnapshotEntry> = data
            .entries
            .into_iter()
            .filter(|e| e.owner == owner)
            .collect();

        if entries.is_empty() {
            return Err(SnapshotError::NoSnapshotForOwner {
                account_id: owner,
                key: epoch_key,
            });
        }
        Ok(entries)
    }

    /// Return `true` if the given epoch key has been committed.
    pub fn is_committed(&self, epoch_key: String) -> bool {
        self.snapshots.get(&epoch_key).map(|d| d.committed).unwrap_or(false)
    }

    // ─── private helpers ───────────────────────────────────────────────────────

    fn require_valid_epoch_key(&self, key: &str) {
        require!(!key.is_empty(), SnapshotError::InvalidEpochKey {
            key: key.to_string()
        });
        // Basic format check: must look like an epoch identifier
        require!(
            key.len() >= 3,
            SnapshotError::InvalidEpochKey {
                key: key.to_string()
            }
        );
    }

    fn require_not_committed(&self, key: &str) {
        if let Some(data) = self.snapshots.get(key) {
            require!(
                !data.committed,
                SnapshotError::EpochAlreadyCommitted {
                    key: key.to_string()
                }
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    fn context(predecessor: AccountId) {
        let builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        testing_env!(builder.build());
    }

    #[test]
    fn test_new() {
        context("alice.testnet".parse().unwrap());
        let contract = SnapshotContract::new();
        assert!(contract.get_snapshot("epoch-2026-08".to_string()).is_none());
    }

    #[test]
    fn test_add_entry_and_get() {
        context("alice.testnet".parse().unwrap());
        let mut contract = SnapshotContract::new();

        let entry = HeroSnapshotEntry::new(
            "hero-1".to_string(),
            "alice.testnet".parse().unwrap(),
            100,
            42,
            "Pilot".to_string(),
        );

        contract.add_entry(entry, "epoch-2026-08".to_string());
        let view = contract.get_snapshot("epoch-2026-08".to_string());
        assert!(view.is_some());
        let view = view.unwrap();
        assert_eq!(view.entries.len(), 1);
        assert_eq!(view.entries[0].token_id, "hero-1");
        assert!(!view.committed);
    }

    #[test]
    fn test_commit_epoch_immutable() {
        context("alice.testnet".parse().unwrap());
        let mut contract = SnapshotContract::new();

        let entry = HeroSnapshotEntry::new(
            "hero-1".to_string(),
            "alice.testnet".parse().unwrap(),
            100,
            42,
            "Pilot".to_string(),
        );

        contract.add_entry(entry, "epoch-2026-08".to_string());
        contract.commit_epoch("epoch-2026-08".to_string());

        let view = contract.get_snapshot("epoch-2026-08".to_string()).unwrap();
        assert!(view.committed);
    }

    #[test]
    fn test_get_owner_snapshot() {
        context("alice.testnet".parse().unwrap());
        let mut contract = SnapshotContract::new();

        contract.add_entry(
            HeroSnapshotEntry::new(
                "hero-1".to_string(),
                "alice.testnet".parse().unwrap(),
                100,
                42,
                "Pilot".to_string(),
            ),
            "epoch-2026-08".to_string(),
        );
        contract.add_entry(
            HeroSnapshotEntry::new(
                "hero-2".to_string(),
                "bob.testnet".parse().unwrap(),
                200,
                42,
                "Engineer".to_string(),
            ),
            "epoch-2026-08".to_string(),
        );

        let entries = contract.get_owner_snapshot(
            "alice.testnet".parse().unwrap(),
            "epoch-2026-08".to_string(),
        );
        assert!(entries.is_ok());
        assert_eq!(entries.unwrap().len(), 1);
    }
}
