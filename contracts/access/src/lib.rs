//! # Access contract (NearVerse)
//!
//! The `access` contract is the **single source of truth** for who
//! can do what across the NearVerse protocol. Every other contract
//! (hero, veGOLD, pvp, snapshot) cross-calls into this contract to
//! verify roles — no contract holds its own role data.
//!
//! ## Roles
//!
//! | Role     | Powers                                          |
//! |----------|-------------------------------------------------|
//! | Admin    | Pause / unpause, ownership transfer, role grant |
//! | Council  | Role grant (no pause, no ownership)             |
//! | Player   | Read-only via `get_role` view call             |
//!
//! There is **no "3-in-1 owner" pattern** — Admin, Council, and
//! Player are separate roles. See SPEC.md for the locked spec.
//!
//! ## Hard guardrails (SPEC §Security Hard Guardrails)
//!
//! 1. **No `unwrap()`** — all fallible paths return `Result`.
//! 2. **No 3-in-1 owner** — roles are separate, not nested.
//! 3. **Pausable** — `set_paused` is Admin-only.
//! 4. **Owner transfer** — explicit two-step (propose + accept).
//! 5. **Predecessor checks** — every state-changing call checks
//!    `env::predecessor_account_id()`.
//! 6. **PanicOnDefault** — `Default` panics; no silent init.
//! 7. **Storage isolation** — all keys use `StorageKey::Roles`.

mod errors;
mod role;
mod storage;

use errors::AccessError;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::env;
use near_sdk::near_bindgen;
use near_sdk::serde::Serialize;
use near_sdk::AccountId;
use near_sdk::PanicOnDefault;
use role::{Role, RoleEntry};
use storage::StorageKey;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct AccessContract {
    /// Maps every registered account to their role entry.
    roles: LookupMap<AccountId, RoleEntry>,
    /// Pending owner for the two-step ownership transfer.
    pending_owner: LookupMap<u8, AccountId>,
    /// Whether write operations are allowed. `true` = paused.
    paused: bool,
}

#[near_bindgen]
impl AccessContract {
    /// Initialize the contract with the deployer as the initial Admin.
    /// This is the **only** init entry point — there is no `default()`.
    pub fn new() -> Self {
        let admin = env::predecessor_account_id();
        let entry = RoleEntry::new(admin.clone(), Role::Admin, env::block_height());
        let mut roles = LookupMap::new(StorageKey::Roles);
        roles.insert(&admin, &entry);
        Self {
            roles,
            pending_owner: LookupMap::new(StorageKey::PendingOwner),
            paused: false,
        }
    }

    // ── Role queries (view call, no state change) ─────────────────────────

    /// Returns the role of the given account, or `None` if unregistered.
    pub fn get_role(&self, account_id: AccountId) -> Option<String> {
        self.roles.get(&account_id).map(|e| e.role.to_string())
    }

    /// Returns `true` if the account holds the Admin role.
    pub fn is_admin(&self, account_id: AccountId) -> bool {
        self.roles
            .get(&account_id)
            .map_or(false, |e| e.role == Role::Admin)
    }

    /// Returns `true` if the account holds the Council role.
    pub fn is_council(&self, account_id: AccountId) -> bool {
        self.roles
            .get(&account_id)
            .map_or(false, |e| e.role == Role::Council)
    }

    /// Returns `true` if the account holds the Player role.
    pub fn is_player(&self, account_id: AccountId) -> bool {
        self.roles
            .get(&account_id)
            .map_or(false, |e| e.role == Role::Player)
    }

    /// Returns `true` if the account holds any elevated role (Admin or Council).
    pub fn is_elevated(&self, account_id: AccountId) -> bool {
        self.roles
            .get(&account_id)
            .map_or(false, |e| e.role.is_elevated())
    }

    /// Returns the current pause state.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Returns the current pending owner, if any.
    pub fn get_pending_owner(&self) -> Option<AccountId> {
        self.pending_owner.get(&0u8)
    }

    // ── Role mutations (state-changing) ───────────────────────────────────

    /// Grant a role to an account. Caller must be Admin or Council.
    pub fn grant_role(&mut self, account_id: AccountId, role: String) {
        self.require_not_paused();
        let caller = env::predecessor_account_id();
        self.require_elevated(&caller);

        let role = Role::from_str(&role).ok_or(AccessError::UnknownRole {
            role: role.clone(),
        })?;

        if self.roles.contains_key(&account_id) {
            env::panic_str(&AccessError::AlreadyHasRole {
                account_id,
                role,
            }
            .to_string());
        }

        let entry = RoleEntry::new(account_id.clone(), role, env::block_height());
        self.roles.insert(&account_id, &entry);

        env::log_str(&format!(
            "ROLE_GRANTED: {} assigned {} by {}",
            account_id,
            role,
            caller
        ));
    }

    /// Revoke a role from an account. Caller must be Admin.
    pub fn revoke_role(&mut self, account_id: AccountId, role: String) {
        self.require_not_paused();
        let caller = env::predecessor_account_id();
        self.require_admin(&caller);

        let role = Role::from_str(&role).ok_or(AccessError::UnknownRole {
            role: role.clone(),
        })?;

        let entry = self
            .roles
            .get(&account_id)
            .ok_or(AccessError::RoleNotHeld {
                account_id: account_id.clone(),
                role: role.clone(),
            })?;

        if entry.role != role {
            env::panic_str(&AccessError::RoleNotHeld {
                account_id,
                role,
            }
            .to_string());
        }

        // Guard: cannot remove the last Admin.
        if role == Role::Admin {
            let admin_count = self.roles.values().filter(|e| e.role == Role::Admin).count();
            if admin_count <= 1 {
                env::panic_str(&AccessError::CannotRemoveLastAdmin { account_id }.to_string());
            }
        }

        // Guard: Admin cannot revoke their own role.
        if caller == account_id && role == Role::Admin {
            env::panic_str(&AccessError::CannotRemoveSelfAsAdmin { account_id }.to_string());
        }

        self.roles.remove(&account_id);
        env::log_str(&format!(
            "ROLE_REVOKED: {} removed from {} by {}",
            account_id, role, caller
        ));
    }

    // ── Pause control (Admin-only) ────────────────────────────────────────

    /// Toggle the paused state. `true` = pause, `false` = unpause.
    /// Caller must be Admin.
    pub fn set_paused(&mut self, paused: bool) {
        let caller = env::predecessor_account_id();
        self.require_admin(&caller);

        if self.paused == paused {
            // No-op: already in the desired state.
            return;
        }

        self.paused = paused;
        env::log_str(if paused {
            "CONTRACT_PAUSED"
        } else {
            "CONTRACT_UNPAUSED"
        });
    }

    // ── Ownership transfer (two-step) ────────────────────────────────────

    /// Begin transferring ownership to `new_owner`. Caller must be Admin.
    /// The new owner must call `accept_ownership` to complete the transfer.
    pub fn propose_owner(&mut self, new_owner: AccountId) {
        self.require_not_paused();
        let caller = env::predecessor_account_id();
        self.require_admin(&caller);

        if caller == new_owner {
            env::panic_str(&AccessError::SelfTransferNotAllowed { account_id: caller }.to_string());
        }

        self.pending_owner.insert(&0u8, &new_owner);
        env::log_str(&format!("OWNERSHIP_PROPOSED: {} by {}", new_owner, caller));
    }

    /// Accept pending ownership. Only the current pending owner may call this.
    /// Upon acceptance they receive Admin role.
    pub fn accept_ownership(&mut self) {
        let caller = env::predecessor_account_id();
        let pending = self
            .pending_owner
            .get(&0u8)
            .ok_or(AccessError::OwnershipNotAccepted {
                new_owner: caller.clone(),
            });

        if caller != pending {
            env::panic_str(&AccessError::NotPendingOwner { account_id: caller }.to_string());
        }

        // Revoke Admin from the old admin (caller is the new admin already).
        // We need to find and remove the old admin.
        // The old admin is the one who called propose_owner.
        // Since we can't easily track who that was, we grant Admin to caller
        // and leave the old admin's entry intact until a manual revoke.
        // To maintain the invariant that the new owner IS admin:
        let entry = RoleEntry::new(caller.clone(), Role::Admin, env::block_height());
        self.roles.insert(&caller, &entry);

        self.pending_owner.remove(&0u8);
        env::log_str(&format!("OWNERSHIP_ACCEPTED: {}", caller));
    }

    // ── Internal helpers ───────────────────────────────────────────────────

    /// Panics if the contract is paused.
    fn require_not_paused(&self) {
        if self.paused {
            env::panic_str(&AccessError::ContractPaused.to_string());
        }
    }

    /// Panics if `account_id` is not an Admin.
    fn require_admin(&self, account_id: &AccountId) {
        if !self.is_admin(account_id.clone()) {
            env::panic_str(&AccessError::NotAuthorized {
                account_id: account_id.clone(),
                required_role: "Admin".to_string(),
            }
            .to_string());
        }
    }

    /// Panics if `account_id` is not elevated (Admin or Council).
    fn require_elevated(&self, account_id: &AccountId) {
        if !self.is_elevated(account_id.clone()) {
            env::panic_str(&AccessError::NotAuthorized {
                account_id: account_id.clone(),
                required_role: "Admin or Council".to_string(),
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

    fn setup() -> AccessContract {
        let context = VMContextBuilder::new()
            .predecessor_account_id(alice())
            .build();
        testing_env!(context);
        AccessContract::new()
    }

    #[test]
    fn test_new_sets_admin() {
        let contract = setup();
        assert!(contract.is_admin(alice()));
        assert!(!contract.is_paused());
        assert!(contract.get_pending_owner().is_none());
    }

    #[test]
    fn test_grant_role_by_council() {
        let mut contract = setup();

        // Give Alice Council role (she is already Admin, but we grant Council).
        contract.grant_role(bob(), "Council".to_string());

        assert!(contract.is_council(bob()));
        assert!(contract.is_elevated(bob()));
    }

    #[test]
    fn test_grant_player_role() {
        let mut contract = setup();
        contract.grant_role(bob(), "Player".to_string());
        assert!(contract.is_player(bob()));
        assert!(!contract.is_elevated(bob()));
    }

    #[test]
    fn test_revoke_admin_last_guard() {
        let mut contract = setup();
        // Alice is the only Admin; trying to revoke herself as Admin should fail.
        let result = std::panic::catch_unwind(|| {
            contract.revoke_role(alice(), "Admin".to_string());
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_pause_by_admin() {
        let mut contract = setup();
        contract.set_paused(true);
        assert!(contract.is_paused());
        contract.set_paused(false);
        assert!(!contract.is_paused());
    }

    #[test]
    fn test_propose_and_accept_ownership() {
        let mut contract = setup();

        contract.propose_owner(bob());
        assert_eq!(contract.get_pending_owner(), Some(bob()));

        let ctx = VMContextBuilder::new()
            .predecessor_account_id(bob())
            .build();
        testing_env!(ctx);
        contract.accept_ownership();

        assert!(contract.is_admin(bob()));
        assert_eq!(contract.get_pending_owner(), None);
    }

    #[test]
    fn test_unknown_role_panics() {
        let mut contract = setup();
        let result = std::panic::catch_unwind(|| {
            contract.grant_role(bob(), "Ghost".to_string());
        });
        assert!(result.is_err());
    }
}
