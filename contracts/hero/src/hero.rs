//! Hero domain types: classes and the hero NFT struct.

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::Serialize;

/// The five hero classes defined in SPEC.md.
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
    /// Parse a string into a HeroClass. Returns `None` for unknown classes.
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

    /// Human-readable name of the class.
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

/// Hero NFT stats. Derived deterministically from `token_id` via
/// `derive_stats()`, guaranteeing consistent on-chain stats with no
/// off-chain randomness.
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
    /// Pure function — no storage reads, no randomness.
    fn derive_from_seed(seed: u64) -> Self {
        let power = 10 + ((seed >> 0) % 91) as u8;
        let speed = 10 + ((seed >> 8) % 91) as u8;
        let defense = 10 + ((seed >> 16) % 91) as u8;
        let stamina = 10 + ((seed >> 24) % 91) as u8;

        Self {
            power,
            speed,
            defense,
            stamina,
        }
    }
}

/// A hero NFT (NEP-171 token).
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct Hero {
    pub token_id: String,
    pub class: HeroClass,
    pub stats: HeroStats,
    pub minted_at: u64,
    pub description: String,
}

impl Hero {
    /// Mint a new hero. Stats are derived deterministically from the token_id.
    pub fn new(token_id: String, class: HeroClass, minted_at: u64) -> Self {
        // Build a seed from the token_id bytes and the mint height.
        let mut seed: u64 = 1469598103934665603; // FNV offset basis
        for b in token_id.bytes() {
            seed = seed.wrapping_mul(1099511628211).wrapping_add(b as u64);
        }
        seed = seed.wrapping_add(minted_at);

        let stats = HeroStats::derive_from_seed(seed);
        let description = format!(
            "{} class hero - power: {}, speed: {}, defense: {}, stamina: {}",
            class.display_name(),
            stats.power,
            stats.speed,
            stats.defense,
            stats.stamina
        );

        Self {
            token_id,
            class,
            stats,
            minted_at,
            description,
        }
    }

    /// Sum of all stats — used by PvP combat math.
    pub fn total_power(&self) -> u32 {
        self.stats.power as u32
            + self.stats.speed as u32
            + self.stats.defense as u32
            + self.stats.stamina as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_from_str() {
        assert_eq!(HeroClass::from_str("Pilot"), Some(HeroClass::Pilot));
        assert_eq!(HeroClass::from_str("Engineer"), Some(HeroClass::Engineer));
        assert_eq!(HeroClass::from_str("Bogus"), None);
    }

    #[test]
    fn test_deterministic_stats() {
        let h1 = Hero::new("hero-1".to_string(), HeroClass::Pilot, 100);
        let h2 = Hero::new("hero-1".to_string(), HeroClass::Pilot, 100);
        assert_eq!(h1.stats.power, h2.stats.power);
        assert_eq!(h1.stats.speed, h2.stats.speed);
    }

    #[test]
    fn test_different_seeds_different_stats() {
        let h1 = Hero::new("hero-1".to_string(), HeroClass::Pilot, 100);
        let h2 = Hero::new("hero-2".to_string(), HeroClass::Pilot, 100);
        let diff = h1.stats.power != h2.stats.power
            || h1.stats.speed != h2.stats.speed
            || h1.stats.defense != h2.stats.defense
            || h1.stats.stamina != h2.stats.stamina;
        assert!(diff, "Two different tokens should have different stats");
    }

    #[test]
    fn test_total_power() {
        let h = Hero::new("test-hero".to_string(), HeroClass::Engineer, 50);
        assert!(h.total_power() >= 40);
        assert!(h.total_power() <= 400);
    }
}
