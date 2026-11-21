//! Progression systems: waves, experience, in-run upgrades, and hangar shop.

pub mod experience;
pub mod shop;
pub mod upgrades;
pub mod waves;

pub use experience::{combo_xp_multiplier, next_level_xp, scaled_shard_pickup, Gem};
pub use shop::{max_level_for_id, SHOP_ITEMS};
pub use upgrades::{roll_three_upgrades, UpgradeCard, ALL_UPGRADES};
pub use waves::{
    boss_count_for_wave, event_for_wave, wave_spawn_rate, DifficultySpec, WaveEvent, DIFFICULTIES,
    WAVE_DURATION,
};
