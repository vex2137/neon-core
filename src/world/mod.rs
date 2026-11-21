//! World systems: arena bounds, spatial hash grid, orbital hazards, and beacon objectives.

pub mod arena;
pub mod hazards;
pub mod objectives;
pub mod spatial_grid;

pub use arena::{clamp_to_arena, generate_starfield, Star};
pub use hazards::{tick_hazard_spawner, HazardZone};
pub use objectives::{BeaconObjective, BeaconTickEvent};
pub use spatial_grid::SpatialGrid;
