use near_sdk::AccountId;
use near_sdk::serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "error", content = "details")]
pub enum HeroError {
    TokenAlreadyExists { token_id: String },
    TokenNotFound { token_id: String },
    NotTokenOwner { account_id: AccountId, token_id: String },
    UnknownClass { class: String },
    ContractPaused,
    InvalidRecipient { recipient: AccountId },
    NotApproved { account_id: AccountId, token_id: String },
    AlreadyApproved { account_id: AccountId, token_id: String },
    NoApproval { token_id: String },
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
            HeroError::NotTokenOwner { account_id, token_id } => write!(
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
            HeroError::NotApproved { account_id, token_id } => write!(
                f,
                "{} is not approved for hero {}",
                account_id, token_id
            ),
            HeroError::AlreadyApproved { account_id, token_id } => write!(
                f,
                "{} is already approved for hero {}",
                account_id, token_id
            ),
            HeroError::NoApproval { token_id } => {
                write!(f, "No approval set for hero {}", token_id)
            }
        }
    }
}
