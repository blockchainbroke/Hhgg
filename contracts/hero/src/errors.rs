//! Error types for the hero NFT contract.

use near_sdk::AccountId;
use near_sdk::serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "error", content = "details")]
pub enum HeroError {
    /// Token ID already exists.
    TokenAlreadyExists { token_id: String },
    /// Token ID was not found.
    TokenNotFound { token_id: String },
    /// Caller is not the owner of the token.
    NotTokenOwner {
        account_id: AccountId,
        token_id: String,
    },
    /// Hero class string is not one of the five valid classes.
    UnknownClass { class: String },
    /// Contract is paused.
    ContractPaused,
    /// Recipient account ID is invalid.
    InvalidRecipient { recipient: AccountId },
    /// Token was not approved for the caller.
    NotApproved { account_id: AccountId, token_id: String },
}

impl std::fmt::Display for HeroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeroError::TokenAlreadyExists { token_id } => {
                write!(f, "Hero with token_id {} already exists", token_id)
            }
            HeroError::TokenNotFound { token_id } => {
                write!(f, "Hero with token_id {} not found", token_id)
            }
            HeroError::NotTokenOwner {
                account_id,
                token_id,
            } => write!(
                f,
                "Account {} is not the owner of hero {}",
                account_id, token_id
            ),
            HeroError::UnknownClass { class } => {
                write!(f, "Unknown hero class: {}", class)
            }
            HeroError::ContractPaused => write!(f, "Hero contract is paused"),
            HeroError::InvalidRecipient { recipient } => {
                write!(f, "Invalid recipient account: {}", recipient)
            }
            HeroError::NotApproved {
                account_id,
                token_id,
            } => write!(
                f,
                "Account {} is not approved to transfer hero {}",
                account_id, token_id
            ),
        }
    }
}
