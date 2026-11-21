//! Enemy subsystem: archetype types, spawning/scaling, and AI/boss phases.

pub mod ai;
pub mod spawning;
pub mod types;

pub use ai::update_enemies_ai;
pub use spawning::{compute_spawn_position, create_enemy, roll_wave_enemy_kind};
pub use types::{Enemy, EnemyKind};
