//! Role type definitions for the access contract.
//!
//! Per SPEC.md the contract enforces three distinct roles - Admin,
//! Council, and Player - without a "3-in-1 owner" pattern.

use near_sdk::AccountId;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use std::fmt;

/// The three protocol roles. Admin and Council are privileged;
/// Player is the base role for ordinary users.
#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
#[borsh(crate = "near_sdk::borsh")]
pub enum Role {
    Admin,
    Council,
    Player,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Admin => write!(f, "Admin"),
            Role::Council => write!(f, "Council"),
            Role::Player => write!(f, "Player"),
        }
    }
}

impl Role {
    /// Returns `true` if this role has elevated (Admin or Council) privileges.
    pub fn is_elevated(&self) -> bool {
        matches!(self, Role::Admin | Role::Council)
    }

    /// Parse a string to a `Role`. Returns `None` for unknown strings.
    pub fn from_str(s: &str) -> Option<Role> {
        match s {
            "Admin" => Some(Role::Admin),
            "Council" => Some(Role::Council),
            "Player" => Some(Role::Player),
            _ => None,
        }
    }
}

/// Snapshot of an account's role registration. Stored in the Roles map.
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct RoleEntry {
    pub account_id: AccountId,
    pub role: Role,
    /// Block height at which the role was registered.
    pub registered_at: u64,
}

impl RoleEntry {
    pub fn new(account_id: AccountId, role: Role, block_height: u64) -> Self {
        Self {
            account_id,
            role,
            registered_at: block_height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_display() {
        assert_eq!(format!("{}", Role::Admin), "Admin");
        assert_eq!(format!("{}", Role::Council), "Council");
        assert_eq!(format!("{}", Role::Player), "Player");
    }

    #[test]
    fn test_role_is_elevated() {
        assert!(Role::Admin.is_elevated());
        assert!(Role::Council.is_elevated());
        assert!(!Role::Player.is_elevated());
    }

    #[test]
    fn test_role_from_str() {
        assert_eq!(Role::from_str("Admin"), Some(Role::Admin));
        assert_eq!(Role::from_str("Council"), Some(Role::Council));
        assert_eq!(Role::from_str("Player"), Some(Role::Player));
        assert_eq!(Role::from_str("Bogus"), None);
    }
}
