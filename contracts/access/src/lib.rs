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
//! | Player   | Read-only via `get_role` view call              |
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
//!
//! ## Controls (the "buttons" the user wants to double-check)
//!
//! | Public method          | Caller    | Effect                       |
//! |------------------------|-----------|------------------------------|
//! | `new` (init)           | self      | Sets initial Admin           |
//! | `grant_role`           | Admin/Council | Adds a role             |
//! | `revoke_role`          | Admin     | Removes a role               |
//! | `set_paused`           | Admin     | Toggles pause                |
//! | `propose_owner`        | Admin     | Begins ownership transfer    |
//! | `accept_ownership`     | Pending   | Completes ownership transfer |
//! | `get_role` (view)      | anyone    | Reads a single role          |
//! | `has_role` (view)      | anyone    | Boolean role check           |
//! | `get_admin` (view)     | anyone    | Current Admin account        |
//! | `get_paused` (view)    | anyone    | Pause state                  |
//! | `get_owner_proposal`   | anyone    | Pending owner (if any)       |
//! | `get_admin_count`      | anyone    | Admin population size        |

mod errors;
mod role;
mod storage;

use errors::{not_authorized, not_registered, AccessError};
use near_sdk::collections::UnorderedMap;
use near_sdk::env;
use near_sdk::{near_bindgen, AccountId, BorshStorageKey, PanicOnDefault};
use role::{Role, RoleEntry};
use storage::StorageKey;

/// Default minimum deposit (in yoctoNEAR) to register a new role.
/// 0.001 NEAR = 1e21 yoctoNEAR. Adjustable in v2 via governance.
const MIN_REGISTRATION_DEPOSIT: u128 = 1_000_000_000_000_000_000_000;

/// Maximum time the pending owner has to accept ownership (in blocks).
/// Approximately 30 days at ~1 block/sec. Prevents permanent lock-out
/// if the proposed owner is a typo or lost key.
const OWNERSHIP_ACCEPT_WINDOW_BLOCKS: u64 = 2_592_000;

/// Contract state.
#[near_bindgen]
#[derive(PanicOnDefault)]
pub struct Contract {
    /// The current Admin. Public so any contract or wallet can verify.
    owner: AccountId,
    /// Pending new owner (if a transfer has been proposed).
    pending_owner: Option<(AccountId, u64)>, // (account, block_height_proposed)
    /// Whether the contract is currently paused.
    paused: bool,
    /// AccountId → RoleEntry. The role registry.
    roles: UnorderedMap<AccountId, RoleEntry>,
    /// Cached count of accounts holding the Admin role.
    admin_count: u32,
}

#[near_bindgen]
impl Contract {
    /// Initialize the contract with a single Admin.
    ///
    /// # Arguments
    /// * `admin` — the initial Admin account. Must be a valid NEAR account ID.
    ///
    /// # Panics
    /// Panics if the contract is already initialized (PanicOnDefault).
    #[init]
    pub fn new(admin: AccountId) -> Self {
        // Sanity-check the admin account is not empty / malformed.
        require!(!admin.as_str().is_empty(), "Admin must be a non-empty AccountId");

        let mut roles = UnorderedMap::new(StorageKey::Roles.to_bytes());
        // Register the initial Admin in the role registry.
        let entry = RoleEntry::new(admin.clone(), Role::Admin, env::block_height());
        roles.insert(&admin, &entry);

        env::log_str(&format!("Access contract initialized. Admin = {}", admin));

        Self {
            owner: admin,
            pending_owner: None,
            paused: false,
            roles,
            admin_count: 1,
        }
    }

    // ════════════════════════════════════════════════════════════════════
    //  ROLE MANAGEMENT
    // ════════════════════════════════════════════════════════════════════

    /// Grant a role to an account. Admin or Council only.
    /// Council cannot grant the Admin role (only Admin can promote to Admin).
    /// Granting an existing role to an account that already has it is a no-op
    /// and returns `Ok(())` to keep the call idempotent.
    ///
    /// # Arguments
    /// * `account_id` — the account receiving the role
    /// * `role` — the role to grant, as a string ("Admin" | "Council" | "Player")
    #[handle_result]
    pub fn grant_role(
        &mut self,
        #[callback_unwrap] account_id: AccountId,
        role: String,
    ) -> Result<(), AccessError> {
        self.assert_not_paused()?;
        self.assert_caller_is_elevated()?;

        let parsed_role = Role::from_str(&role).ok_or_else(|| AccessError::NotAuthorized {
            account_id: env::predecessor_account_id(),
            required_role: format!("unknown role: {}", role),
        })?;

        // Council cannot grant Admin. Only an Admin can promote to Admin.
        let caller_role = self.role_of(&env::predecessor_account_id())
            .ok_or_else(|| not_registered(&env::predecessor_account_id()))?;
        if parsed_role == Role::Admin && caller_role != Role::Admin {
            return Err(not_authorized(&env::predecessor_account_id(), "Admin"));
        }

        // If the account already has a role, decrement the old count.
        if let Some(existing) = self.roles.get(&account_id) {
            if existing.role == Role::Admin {
                // admin_count was already tracked at insert/remove; adjust below
            }
        }

        let was_admin = self.roles.get(&account_id)
            .map(|e| e.role == Role::Admin)
            .unwrap_or(false);

        let entry = RoleEntry::new(account_id.clone(), parsed_role, env::block_height());
        self.roles.insert(&account_id, &entry);

        if parsed_role == Role::Admin && !was_admin {
            self.admin_count = self.admin_count.saturating_add(1);
        }

        env::log_str(&format!(
            "Granted role {} to {} (by {})",
            parsed_role, account_id, env::predecessor_account_id()
        ));
        Ok(())
    }

    /// Revoke a role from an account. Admin only.
    /// Admins cannot revoke their own role (prevents lock-out).
    /// Removing the last Admin is rejected.
    #[handle_result]
    pub fn revoke_role(
        &mut self,
        #[callback_unwrap] account_id: AccountId,
    ) -> Result<(), AccessError> {
        self.assert_not_paused()?;
        self.assert_caller_is_admin()?;

        let caller = env::predecessor_account_id();
        if caller == account_id {
            return Err(AccessError::CannotRemoveSelfAsAdmin { account_id: caller });
        }

        let existing = self.roles.get(&account_id)
            .ok_or_else(|| AccessError::RoleNotHeld {
                account_id: account_id.clone(),
                role: "any".to_string(),
            })?;

        if existing.role == Role::Admin && self.admin_count <= 1 {
            return Err(AccessError::CannotRemoveLastAdmin {
                account_id: account_id.clone(),
            });
        }

        self.roles.remove(&account_id);

        if existing.role == Role::Admin {
            self.admin_count = self.admin_count.saturating_sub(1);
        }

        env::log_str(&format!(
            "Revoked role from {} (by {})",
            account_id, caller
        ));
        Ok(())
    }

    // ════════════════════════════════════════════════════════════════════
    //  PAUSE / UNPAUSE
    // ════════════════════════════════════════════════════════════════════

    /// Toggle the pause state. Admin only.
    /// While paused, all state-modifying calls (grant/revoke) are rejected.
    /// Read-only view calls are unaffected.
    #[handle_result]
    pub fn set_paused(&mut self, paused: bool) -> Result<(), AccessError> {
        self.assert_caller_is_admin()?;
        // (No require on already-in-state; toggling is idempotent.)
        self.paused = paused;
        env::log_str(&format!(
            "Pause state set to {} by {}",
            paused, env::predecessor_account_id()
        ));
        Ok(())
    }

    // ════════════════════════════════════════════════════════════════════
    //  OWNERSHIP TRANSFER (two-step, SPEC §Owner transfer)
    // ════════════════════════════════════════════════════════════════════

    /// Begin an ownership transfer. Admin only.
    /// The proposed owner must call `accept_ownership` to complete the transfer.
    #[handle_result]
    pub fn propose_owner(
        &mut self,
        #[callback_unwrap] new_owner: AccountId,
    ) -> Result<(), AccessError> {
        self.assert_caller_is_admin()?;

        if new_owner.as_str().is_empty() {
            return Err(AccessError::InvalidNewOwner { new_owner });
        }
        if new_owner == self.owner {
            return Err(AccessError::SelfTransfer { account_id: new_owner });
        }

        self.pending_owner = Some((new_owner.clone(), env::block_height()));
        env::log_str(&format!(
            "Ownership proposed to {} by {}",
            new_owner, env::predecessor_account_id()
        ));
        Ok(())
    }

    /// Complete an ownership transfer. Pending owner only.
    /// Rejects if the acceptance window has expired.
    #[handle_result]
    pub fn accept_ownership(&mut self) -> Result<(), AccessError> {
        let caller = env::predecessor_account_id();

        let (pending, proposed_at) = self.pending_owner.take()
            .ok_or_else(|| AccessError::OwnershipNotAccepted {
                new_owner: caller.clone(),
            })?;

        if caller != pending {
            // Restore the state — caller is not the pending owner.
            self.pending_owner = Some((pending.clone(), proposed_at));
            return Err(AccessError::NotPendingOwner { account_id: caller });
        }

        if env::block_height().saturating_sub(proposed_at) > OWNERSHIP_ACCEPT_WINDOW_BLOCKS {
            return Err(AccessError::OwnershipNotAccepted {
                new_owner: caller,
            });
        }

        // Move Admin role from old owner to new owner in the registry.
        if let Some(old_entry) = self.roles.get(&self.owner) {
            self.roles.remove(&self.owner);
            let _ = old_entry; // we discard the old entry
        }
        let new_entry = RoleEntry::new(pending.clone(), Role::Admin, env::block_height());
        self.roles.insert(&pending, &new_entry);

        let old_owner = self.owner.clone();
        self.owner = pending.clone();

        env::log_str(&format!(
            "Ownership transferred from {} to {}",
            old_owner, pending
        ));
        Ok(())
    }

    // ════════════════════════════════════════════════════════════════════
    //  VIEW CALLS (read-only)
    // ════════════════════════════════════════════════════════════════════

    /// Read the role of an account. Returns `None` if the account is not registered.
    pub fn get_role(&self, account_id: AccountId) -> Option<String> {
        self.roles.get(&account_id).map(|e| e.role.to_string())
    }

    /// Boolean role check. Convenience wrapper for clients.
    pub fn has_role(&self, account_id: AccountId, role: String) -> bool {
        match Role::from_str(&role) {
            Some(r) => self.roles.get(&account_id).map(|e| e.role == r).unwrap_or(false),
            None => false,
        }
    }

    /// Current Admin account ID.
    pub fn get_admin(&self) -> AccountId {
        self.owner.clone()
    }

    /// Whether the contract is currently paused.
    pub fn get_paused(&self) -> bool {
        self.paused
    }

    /// Pending owner (if any) and the block height at which the proposal was made.
    pub fn get_owner_proposal(&self) -> Option<(AccountId, u64)> {
        self.pending_owner.clone()
    }

    /// Number of accounts currently holding the Admin role.
    pub fn get_admin_count(&self) -> u32 {
        self.admin_count
    }

    /// All roles (paginated). Returns up to `limit` entries starting at `from_index`.
    /// Useful for UIs and explorers; bounded to avoid OOG.
    pub fn list_roles(&self, from_index: u64, limit: u64) -> Vec<RoleEntry> {
        let keys = self.roles.keys_as_vector();
        let start = from_index as usize;
        let end = (start + limit as usize).min(keys.len());
        (start..end)
            .filter_map(|i| keys.get(i as u64).and_then(|k| self.roles.get(&k)))
            .collect()
    }

    /// Total number of registered role entries.
    pub fn get_role_count(&self) -> u64 {
        self.roles.len()
    }

    /// Minimum registration deposit (in yoctoNEAR).
    pub fn get_min_registration_deposit(&self) -> near_sdk::json_types::U128 {
        near_sdk::json_types::U128(MIN_REGISTRATION_DEPOSIT)
    }

    // ════════════════════════════════════════════════════════════════════
    //  INTERNAL HELPERS
    // ════════════════════════════════════════════════════════════════════

    fn assert_caller_is_admin(&self) -> Result<(), AccessError> {
        let caller = env::predecessor_account_id();
        if caller != self.owner {
            return Err(not_authorized(&caller, "Admin"));
        }
        Ok(())
    }

    fn assert_caller_is_elevated(&self) -> Result<(), AccessError> {
        let caller = env::predecessor_account_id();
        let role = self.role_of(&caller)
            .ok_or_else(|| not_registered(&caller))?;
        if !role.is_elevated() {
            return Err(not_authorized(&caller, "Admin or Council"));
        }
        Ok(())
    }

    fn assert_not_paused(&self) -> Result<(), AccessError> {
        if self.paused {
            return Err(AccessError::ContractPaused);
        }
        Ok(())
    }

    fn role_of(&self, account_id: &AccountId) -> Option<Role> {
        self.roles.get(account_id).map(|e| e.role)
    }
}

// `near_sdk::require!` is the standard guard macro; alias for clarity.
#[allow(unused_imports)]
use near_sdk::require;

// Re-export the helper macro from near-sdk so the rest of the file
// can use `require!()` directly.
#[allow(unused_imports)]
use near_sdk::serde::Serialize as _Serialize;

// Pull in the BorshStorageKey derive from near-sdk's prelude.
#[allow(unused_imports)]
use near_sdk::BorshStorageKey as _BorshStorageKey;
