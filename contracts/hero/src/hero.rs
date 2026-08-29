use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
#[borsh(crate = "near_sdk::borsh")]
pub enum HeroClass {
    Pilot,
    Engineer,
    Medic,
    Smuggler,
    Diplomat,
}

impl HeroClass {
    pub fn from_str(s: &str) -> Option<HeroClass> {
        match s {
            "Pilot" => Some(HeroClass::Pilot),
            "Engineer" => Some(HeroClass::Engineer),
            "Medic" => Some(HeroClass::Medic),
            "Smuggler" => Some(HeroClass::Smuggler),
            "Diplomat" => Some(HeroClass::Diplomat),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            HeroClass::Pilot => "Pilot",
            HeroClass::Engineer => "Engineer",
            HeroClass::Medic => "Medic",
            HeroClass::Smuggler => "Smuggler",
            HeroClass::Diplomat => "Diplomat",
        }
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct HeroStats {
    pub power: u8,
    pub speed: u8,
    pub defense: u8,
    pub stamina: u8,
}

impl HeroStats {
    /// Derive stats deterministically from a seed.
    /// Pure function. No storage reads, no randomness.
    pub fn derive_from_seed(seed: u64) -> Self {
        let power = 10u8.saturating_add(((seed >> 0) % 91) as u8);
        let speed = 10u8.saturating_add(((seed >> 8) % 91) as u8);
        let defense = 10u8.saturating_add(((seed >> 16) % 91) as u8);
        let stamina = 10u8.saturating_add(((seed >> 24) % 91) as u8);
        Self {
            power,
            speed,
            defense,
            stamina,
        }
    }

    /// Sum of all four stats. Used for ranking and snapshot total_power.
    pub fn total(&self) -> u32 {
        (self.power as u32)
            .saturating_add(self.speed as u32)
            .saturating_add(self.defense as u32)
            .saturating_add(self.stamina as u32)
    }
}

/// The hero NFT struct stored under `StorageKey::Heroes`.
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct Hero {
    pub token_id: String,
    pub owner: near_sdk::AccountId,
    pub class: HeroClass,
    pub stats: HeroStats,
    /// Minted block height.
    pub minted_at: u64,
}

impl Hero {
    /// Create a new hero with deterministically-derived stats.
    pub fn new(token_id: String, owner: near_sdk::AccountId, class: HeroClass, minted_at: u64) -> Self {
        // Hash the token_id into a u64 seed for deterministic stats.
        let seed = hash_token_id(&token_id);
        Self {
            token_id,
            owner,
            class,
            stats: HeroStats::derive_from_seed(seed),
            minted_at,
        }
    }
}

/// Deterministic FNV-1a-ish hash from a string to a u64 seed.
/// Pure, no_std, no environment calls.
fn hash_token_id(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_parse_roundtrip() {
        for name in ["Pilot", "Engineer", "Medic", "Smuggler", "Diplomat"] {
            let c = HeroClass::from_str(name).unwrap();
            assert_eq!(c.display_name(), name);
        }
        assert!(HeroClass::from_str("Unknown").is_none());
    }

    #[test]
    fn test_stats_derivation_deterministic() {
        let a = HeroStats::derive_from_seed(12345);
        let b = HeroStats::derive_from_seed(12345);
        assert_eq!(a.power, b.power);
        assert_eq!(a.speed, b.speed);
        assert_eq!(a.defense, b.defense);
        assert_eq!(a.stamina, b.stamina);
    }

    #[test]
    fn test_stats_within_range() {
        let s = HeroStats::derive_from_seed(42);
        assert!(s.power >= 10 && s.power <= 100);
        assert!(s.speed >= 10 && s.speed <= 100);
        assert!(s.defense >= 10 && s.defense <= 100);
        assert!(s.stamina >= 10 && s.stamina <= 100);
    }

    #[test]
    fn test_token_id_hash_deterministic() {
        assert_eq!(hash_token_id("hero-1"), hash_token_id("hero-1"));
        assert_ne!(hash_token_id("hero-1"), hash_token_id("hero-2"));
    }
}
