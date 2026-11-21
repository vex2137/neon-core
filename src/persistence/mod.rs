//! Save persistence and format migration.

#[cfg(any(not(target_arch = "wasm32"), test))]
pub mod migration;
pub mod save;

pub use save::MetaSave;
