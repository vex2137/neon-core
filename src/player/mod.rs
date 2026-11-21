//! Player subsystem: hull specs, combat stats, movement, aiming/weapons, and abilities.

pub mod abilities;
pub mod movement;
pub mod stats;
pub mod weapons;

pub use abilities::try_trigger_dash;
pub use movement::update_player_movement;
pub use stats::{ship_by_id, PlayerState, ShipSpec, SHIPS};
pub use weapons::{update_aim, update_primary_weapon};
