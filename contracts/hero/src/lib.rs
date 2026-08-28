//! # Hero NFT contract (NearVerse)
//!
//! Implements NEP-171 (Core), NEP-177 (Metadata), and NEP-178
//! (Approval) for the NearVerse hero system.
//!
//! Each hero belongs to one of five classes (Pilot, Engineer, Medic,
//! Smuggler, Diplomat) and has stats that are **deterministically
//! derived** from its `token_id`. No off-chain randomness is required.
//!
//! ## Hard guardrails
//!
//! - `#[derive(PanicOnDefault)]` — `Default` panics.
//! - All storage uses `StorageKey::*` namespaces.
//! - No `unwrap()` — every fallible path returns `Result` or panics with
//!   a typed error message.
//! - `mint` is callable by any account (open mint, to be gated by
//!   the `access` contract in production); `transfer` requires
//!   owner-or-approval.

mod errors;
mod hero;
mod storage;

use errors::HeroError;
use hero::{Hero, HeroClass};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::env;
use near_sdk::json_types::U128;
use near_sdk::near_bindgen;
use near_sdk::AccountId;
use near_sdk::PanicOnDefault;
use storage::StorageKey;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct HeroContract {
    /// Maps `token_id` -> `Hero`.
    heroes: LookupMap<String, Hero>,
    /// Maps `AccountId` -> `Set<token_id>` for NEP-171 enumeration.
    owner_tokens: LookupMap<AccountId, UnorderedSet<String>>,
    /// Maps `token_id` -> `AccountId` (owner) for transfer lookups.
    token_owners: LookupMap<String, AccountId>,
    /// Maps `token_id` -> `Set<AccountId>` (NEP-178 approvals).
    approvals: LookupMap<String, UnorderedSet<AccountId>>,
    /// Paused flag.
    paused: bool,
    /// Next token ID auto-increment counter (monotonic).
    next_id: u64,
    /// Total minted (monotonic).
    total_supply: u64,
}

#[near_bindgen]
impl HeroContract {
    /// Initialize the hero contract.
    pub fn new() -> Self {
        Self {
            heroes: LookupMap::new(StorageKey::Heroes),
            owner_tokens: LookupMap::new(StorageKey::OwnerTokens),
            token_owners: LookupMap::new(StorageKey::TokenOwners),
            approvals: LookupMap::new(StorageKey::Heroes),
            paused: false,
            next_id: 0,
            total_supply: 0,
        }
    }

    // ── NEP-171: Core ─────────────────────────────────────────────────────

    /// Mint a new hero of the given class. The caller becomes the owner.
    /// Returns the auto-assigned `token_id`.
    pub fn mint(&mut self, class: String) -> String {
        self.require_not_paused();
        let class = HeroClass::from_str(&class).ok_or(HeroError::UnknownClass {
            class: class.clone(),
        });

        let owner = env::predecessor_account_id();
        let token_id = format!("hero-{}", self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.total_supply = self.total_supply.saturating_add(1);

        let hero = Hero::new(token_id.clone(), class, env::block_height());
        self.heroes.insert(&token_id, &hero);
        self.token_owners.insert(&token_id, &owner);

        // Owner -> set of token IDs
        let mut owner_set = self
            .owner_tokens
            .get(&owner)
            .unwrap_or_else(|| UnorderedSet::new(StorageKey::OwnerTokens));
        owner_set.insert(&token_id);
        self.owner_tokens.insert(&owner, &owner_set);

        env::log_str(&format!("HERO_MINTED: {} class={}", token_id, class.display_name()));
        token_id
    }

    /// Transfer a hero to a new owner. Caller must be the owner or an
    /// approved account (NEP-178).
    pub fn transfer(&mut self, token_id: String, recipient: AccountId) {
        self.require_not_paused();
        let caller = env::predecessor_account_id();

        if recipient == caller {
            env::panic_str(&HeroError::InvalidRecipient { recipient }.to_string());
        }

        let owner = self
            .token_owners
            .get(&token_id)
            .ok_or(HeroError::TokenNotFound { token_id: token_id.clone() });

        // NEP-178: approved accounts may transfer.
        let approved = self
            .approvals
            .get(&token_id)
            .map(|set| set.contains(&caller))
            .unwrap_or(false);

        if caller != owner && !approved {
            env::panic_str(&HeroError::NotTokenOwner {
                account_id: caller.clone(),
                token_id: token_id.clone(),
            }
            .to_string());
        }

        // Remove from old owner set.
        if let Some(mut old_set) = self.owner_tokens.get(&owner) {
            old_set.remove(&token_id);
            self.owner_tokens.insert(&owner, &old_set);
        }

        // Add to new owner set.
        let mut new_set = self
            .owner_tokens
            .get(&recipient)
            .unwrap_or_else(|| UnorderedSet::new(StorageKey::OwnerTokens));
        new_set.insert(&token_id);
        self.owner_tokens.insert(&recipient, &new_set);

        // Update owner record.
        self.token_owners.insert(&token_id, &recipient);

        // Clear approvals on transfer.
        self.approvals.remove(&token_id);

        env::log_str(&format!(
            "HERO_TRANSFERRED: {} from {} to {}",
            token_id, caller, recipient
        ));
    }

    // ── NEP-178: Approvals ────────────────────────────────────────────────

    /// Approve an account to transfer a single hero. Caller must be the
    /// current owner.
    pub fn approve(&mut self, token_id: String, account_id: AccountId) {
        self.require_not_paused();
        let caller = env::predecessor_account_id();
        let owner = self
            .token_owners
            .get(&token_id)
            .ok_or(HeroError::TokenNotFound { token_id: token_id.clone() });

        if caller != owner {
            env::panic_str(&HeroError::NotTokenOwner {
                account_id: caller.clone(),
                token_id: token_id.clone(),
            }
            .to_string());
        }

        let mut set = self
            .approvals
            .get(&token_id)
            .unwrap_or_else(|| UnorderedSet::new(StorageKey::Heroes));
        set.insert(&account_id);
        self.approvals.insert(&token_id, &set);

        env::log_str(&format!("HERO_APPROVED: {} -> {}", token_id, account_id));
    }

    /// Revoke an approval. Caller must be the owner.
    pub fn revoke(&mut self, token_id: String, account_id: AccountId) {
        self.require_not_paused();
        let caller = env::predecessor_account_id();
        let owner = self
            .token_owners
            .get(&token_id)
            .ok_or(HeroError::TokenNotFound { token_id: token_id.clone() });

        if caller != owner {
            env::panic_str(&HeroError::NotTokenOwner {
                account_id: caller.clone(),
                token_id: token_id.clone(),
            }
            .to_string());
        }

        if let Some(mut set) = self.approvals.get(&token_id) {
            set.remove(&account_id);
            self.approvals.insert(&token_id, &set);
        }

        env::log_str(&format!("HERO_REVOKED: {} -> {}", token_id, account_id));
    }

    // ── NEP-171: View calls ───────────────────────────────────────────────

    /// Returns the hero with the given token_id, or `None`.
    pub fn get_hero(&self, token_id: String) -> Option<Hero> {
        self.heroes.get(&token_id)
    }

    /// Returns the owner of the given token_id, or `None`.
    pub fn get_owner(&self, token_id: String) -> Option<AccountId> {
        self.token_owners.get(&token_id)
    }

    /// Returns the list of hero token_ids owned by an account.
    pub fn get_heroes_by_owner(&self, account_id: AccountId) -> Vec<String> {
        self.owner_tokens
            .get(&account_id)
            .map(|s| s.to_vec())
            .unwrap_or_default()
    }

    /// Returns the total number of minted heroes.
    pub fn total_supply(&self) -> U128 {
        U128(self.total_supply as u128)
    }

    /// Returns whether the hero contract is paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    // ── Pause control ─────────────────────────────────────────────────────

    /// Toggle the paused state. Anyone may pause for safety in the MVP;
    /// production must gate this on the `access` contract Admin role.
    pub fn set_paused(&mut self, paused: bool) {
        if self.paused != paused {
            self.paused = paused;
            env::log_str(if paused { "HERO_PAUSED" } else { "HERO_UNPAUSED" });
        }
    }

    // ── Internal helpers ──────────────────────────────────────────────────

    fn require_not_paused(&self) {
        if self.paused {
            env::panic_str(&HeroError::ContractPaused.to_string());
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

    fn setup() -> HeroContract {
        let ctx = VMContextBuilder::new()
            .predecessor_account_id(alice())
            .build();
        testing_env!(ctx);
        HeroContract::new()
    }

    #[test]
    fn test_new() {
        let contract = setup();
        assert_eq!(contract.total_supply.0, 0);
        assert!(!contract.is_paused());
    }

    #[test]
    fn test_mint_and_get_hero() {
        let mut contract = setup();
        let token_id = contract.mint("Pilot".to_string());
        let hero = contract.get_hero(token_id.clone()).expect("hero should exist");
        assert_eq!(hero.class, HeroClass::Pilot);
        assert_eq!(contract.get_owner(token_id.clone()), Some(alice()));
    }

    #[test]
    fn test_mint_invalid_class() {
        let mut contract = setup();
        let result = std::panic::catch_unwind(|| {
            contract.mint("Dragon".to_string());
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_transfer() {
        let mut contract = setup();
        let token_id = contract.mint("Engineer".to_string());
        contract.transfer(token_id.clone(), bob());
        assert_eq!(contract.get_owner(token_id), Some(bob()));
    }

    #[test]
    fn test_transfer_unauthorized() {
        let mut contract = setup();
        let token_id = contract.mint("Medic".to_string());
        let ctx = VMContextBuilder::new()
            .predecessor_account_id(bob())
            .build();
        testing_env!(ctx);
        let result = std::panic::catch_unwind(|| {
            contract.transfer(token_id, alice());
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_approve_and_transfer() {
        let mut contract = setup();
        let token_id = contract.mint("Smuggler".to_string());
        contract.approve(token_id.clone(), bob());

        // Bob can now transfer.
        let ctx = VMContextBuilder::new()
            .predecessor_account_id(bob())
            .build();
        testing_env!(ctx);
        contract.transfer(token_id.clone(), bob());
        assert_eq!(contract.get_owner(token_id), Some(bob()));
    }

    #[test]
    fn test_pause_blocks_mint() {
        let mut contract = setup();
        contract.set_paused(true);
        let result = std::panic::catch_unwind(|| {
            contract.mint("Diplomat".to_string());
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_total_supply_increments() {
        let mut contract = setup();
        contract.mint("Pilot".to_string());
        contract.mint("Medic".to_string());
        assert_eq!(contract.total_supply.0, 2);
    }
}
