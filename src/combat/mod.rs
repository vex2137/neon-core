//! Combat subsystem: projectiles, damage/crit math, and collision updates.

pub mod collisions;
pub mod damage;
pub mod projectiles;

pub use collisions::{update_enemy_bullets, update_gems, update_hazards};
pub use damage::{apply_player_damage, roll_hit_damage, roll_kill_reward};
pub use projectiles::{Bullet, EnemyBullet, Mine};
