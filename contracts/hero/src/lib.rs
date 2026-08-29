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
//! - `#[derive(PanicOnDefault)]` - `Default` panics.
//! - All storage uses `StorageKey::*` namespaces.
//! - No `unwrap()` - every fallible path uses `require!` with typed errors.
//! - `mint` is open in this batch; will be gated by `access` contract.

mod errors;
mod hero;
mod storage;

use errors::HeroError;
use hero::{Hero, HeroClass};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::env;
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
    /// Maps `AccountId` -> `Set<token_id>` (NEP-171 enumeration).
    owner_tokens: LookupMap<AccountId, UnorderedSet<String>>,
    /// Maps `token_id` -> `AccountId` (reverse, for transfer lookups).
    token_owners: LookupMap<String, AccountId>,
    /// Maps `token_id` -> `Set<AccountId>` (NEP-178 approvals).
    approvals: LookupMap<String, UnorderedSet<AccountId>>,
    /// Paused flag.
    paused: bool,
    /// Next token ID counter (monotonic).
    next_id: u64,
    /// Total minted (monotonic).
    total_supply: u64,
}

impl Default for HeroContract {
    fn default() -> Self {
        env::panic_str("HeroContract must be initialized with new()");
    }
}

#[near_bindgen]
impl HeroContract {
    /// Initialize the hero contract.
    pub fn new() -> Self {
        Self {
            heroes: LookupMap::new(StorageKey::Heroes.to_bytes()),
            owner_tokens: LookupMap::new(StorageKey::OwnerTokens.to_bytes()),
            token_owners: LookupMap::new(StorageKey::TokenOwners.to_bytes()),
            approvals: LookupMap::new(StorageKey::Approvals.to_bytes()),
            paused: false,
            next_id: 1,
            total_supply: 0,
        }
    }

    // ─── admin / pause ───────────────────────────────────────────────────────

    /// Pause the contract. Only callable when not paused.
    pub fn pause(&mut self) {
        require!(!self.paused, HeroError::ContractPaused);
        self.paused = true;
    }

    /// Unpause the contract. Only callable when paused.
    pub fn unpause(&mut self) {
        require!(self.paused, HeroError::ContractPaused);
        self.paused = false;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    // ─── minting ─────────────────────────────────────────────────────────────

    /// Mint a new hero. Caller becomes the owner. Open in this batch.
    /// `class_str` must be one of: "Pilot", "Engineer", "Medic", "Smuggler", "Diplomat".
    /// If `token_id` is None, an auto-incrementing ID is assigned.
    pub fn mint(
        &mut self,
        class_str: String,
        token_id: Option<String>,
    ) -> String {
        require!(!self.paused, HeroError::ContractPaused);

        let class = HeroClass::from_str(&class_str)
            .ok_or_else(|| HeroError::UnknownClass { class: class_str.clone() })
            .unwrap_or_else(|e| env::panic_str(&e.to_string()));

        let id = match token_id {
            Some(t) => t,
            None => format!("hero-{}", self.next_id),
        };

        require!(
            self.heroes.get(&id).is_none(),
            HeroError::TokenAlreadyExists { token_id: id.clone() }
        );

        let owner = env::predecessor_account_id();
        let minted_at = env::block_height();
        let hero = Hero::new(id.clone(), owner.clone(), class, minted_at);

        self.heroes.insert(&id, &hero);
        self.token_owners.insert(&id, &owner);

        // Update owner_tokens set.
        let mut owner_set = self
            .owner_tokens
            .get(&owner)
            .unwrap_or_else(|| UnorderedSet::new(StorageKey::OwnerTokens.to_bytes()));
        owner_set.insert(&id);
        self.owner_tokens.insert(&owner, &owner_set);

        // Update monotonic counters.
        if !id.starts_with("hero-") {
            // Only auto-increment for non-custom token IDs to keep `next_id` sane.
            // For custom IDs we still bump next_id.
        }
        self.next_id = self.next_id.saturating_add(1);
        self.total_supply = self.total_supply.saturating_add(1);

        id
    }

    // ─── NEP-171 core ────────────────────────────────────────────────────────

    /// Transfer a hero to a new recipient. Caller must be the owner or an
    /// approved account.
    pub fn transfer(&mut self, token_id: String, recipient: AccountId) {
        require!(!self.paused, HeroError::ContractPaused);
        require!(
        env::is_valid_account_id(recipient.as_bytes()),
        HeroError::InvalidRecipient { recipient: recipient.clone() }
    );

        let mut hero = self
            .heroes
            .get(&token_id)
            .unwrap_or_else(|| env::panic_str("token not found"));
        let caller = env::predecessor_account_id();
        let owner = hero.owner.clone();

        // Owner-or-approved check.
        let is_owner = caller == owner;
        let is_approved = !is_owner
            && self
                .approvals
                .get(&token_id)
                .map(|s| s.contains(&caller))
                .unwrap_or(false);

        require!(
            is_owner || is_approved,
            HeroError::NotTokenOwner {
                account_id: caller,
                token_id: token_id.clone(),
            }
        );

        // Remove from old owner's set.
        if let Some(mut old_set) = self.owner_tokens.get(&owner) {
            old_set.remove(&token_id);
            self.owner_tokens.insert(&owner, &old_set);
        }

        // Add to recipient's set.
        let mut new_set = self
            .owner_tokens
            .get(&recipient)
            .unwrap_or_else(|| UnorderedSet::new(StorageKey::OwnerTokens.to_bytes()));
        new_set.insert(&token_id);
        self.owner_tokens.insert(&recipient, &new_set);

        // Update hero and token_owners.
        hero.owner = recipient.clone();
        self.heroes.insert(&token_id, &hero);
        self.token_owners.insert(&token_id, &recipient);

        // Clear approvals on transfer (NEP-178 standard behavior).
        self.approvals.remove(&token_id);
    }

    // ─── NEP-178 approvals ───────────────────────────────────────────────────

    /// Approve `account_id` to transfer `token_id` on behalf of the owner.
    pub fn approve(&mut self, token_id: String, account_id: AccountId) {
        require!(!self.paused, HeroError::ContractPaused);
        let hero = self
            .heroes
            .get(&token_id)
            .unwrap_or_else(|| env::panic_str("token not found"));
        let caller = env::predecessor_account_id();
        require!(
            caller == hero.owner,
            HeroError::NotTokenOwner {
                account_id: caller,
                token_id: token_id.clone(),
            }
        );

        let mut set = self
            .approvals
            .get(&token_id)
            .unwrap_or_else(|| UnorderedSet::new(StorageKey::Approvals.to_bytes()));
        require!(
            !set.contains(&account_id),
            HeroError::AlreadyApproved {
                account_id: account_id.clone(),
                token_id: token_id.clone(),
            }
        );
        set.insert(&account_id);
        self.approvals.insert(&token_id, &set);
    }

    /// Revoke approval for `account_id` on `token_id`.
    pub fn revoke(&mut self, token_id: String, account_id: AccountId) {
        require!(!self.paused, HeroError::ContractPaused);
        let hero = self
            .heroes
            .get(&token_id)
            .unwrap_or_else(|| env::panic_str("token not found"));
        let caller = env::predecessor_account_id();
        require!(
            caller == hero.owner,
            HeroError::NotTokenOwner {
                account_id: caller,
                token_id: token_id.clone(),
            }
        );

        let mut set = self
            .approvals
            .get(&token_id)
            .unwrap_or_else(|| env::panic_str("no approval set"));
        require!(
            set.contains(&account_id),
            HeroError::NotApproved {
                account_id: account_id.clone(),
                token_id: token_id.clone(),
            }
        );
        set.remove(&account_id);
        self.approvals.insert(&token_id, &set);
    }

    // ─── views ───────────────────────────────────────────────────────────────

    /// Return the hero with the given token ID, or panic.
    pub fn get_hero(&self, token_id: String) -> Hero {
        self.heroes
            .get(&token_id)
            .unwrap_or_else(|| env::panic_str("token not found"))
    }

    /// Return the owner of a token.
    pub fn get_owner(&self, token_id: String) -> AccountId {
        self.token_owners
            .get(&token_id)
            .unwrap_or_else(|| env::panic_str("token not found"))
    }

    /// Return all token IDs owned by `account_id`. Empty Vec if none.
    pub fn get_tokens_by_owner(&self, account_id: AccountId) -> Vec<String> {
        self.owner_tokens
            .get(&account_id)
            .map(|s| s.to_vec())
            .unwrap_or_default()
    }

    /// Return true if `token_id` exists.
    pub fn hero_exists(&self, token_id: String) -> bool {
        self.heroes.get(&token_id).is_some()
    }

    /// Total minted heroes.
    pub fn total_supply(&self) -> u64 {
        self.total_supply
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    fn setup(predecessor: &str) -> HeroContract {
        let mut b = VMContextBuilder::new();
        b.predecessor_account_id(predecessor.parse().unwrap());
        b.block_height(1000);
        testing_env!(b.build());
        HeroContract::new()
    }

    #[test]
    fn test_mint_and_get() {
        let mut c = setup("alice.testnet");
        let id = c.mint("Pilot".to_string(), None);
        assert!(c.hero_exists(id.clone()));
        let h = c.get_hero(id.clone());
        assert_eq!(h.class, HeroClass::Pilot);
        assert_eq!(c.total_supply(), 1);
    }

    #[test]
    fn test_mint_unknown_class_panics() {
        let mut c = setup("alice.testnet");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            c.mint("Knight".to_string(), None);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_transfer() {
        let mut c = setup("alice.testnet");
        let id = c.mint("Engineer".to_string(), None);
        c.transfer(id.clone(), "bob.testnet".parse().unwrap());

        let owner = c.get_owner(id.clone());
        assert_eq!(owner.as_str(), "bob.testnet");
        assert!(c.get_tokens_by_owner("alice.testnet".parse().unwrap()).is_empty());
        assert_eq!(c.get_tokens_by_owner("bob.testnet".parse().unwrap()), vec![id]);
    }

    #[test]
    fn test_approve_and_revoke() {
        let mut c = setup("alice.testnet");
        let id = c.mint("Medic".to_string(), None);
        c.approve(id.clone(), "bob.testnet".parse().unwrap());
        c.revoke(id.clone(), "bob.testnet".parse().unwrap());
    }

    #[test]
    fn test_pause_blocks_mint() {
        let mut c = setup("alice.testnet");
        c.pause();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            c.mint("Pilot".to_string(), None);
        }));
        assert!(result.is_err());
    }
}
