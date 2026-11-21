//! Save persistence, mid-run snapshots, and format migration.

#[cfg(any(not(target_arch = "wasm32"), test))]
pub mod migration;
pub mod run_save;
pub mod save;

pub use run_save::{RunSnapshot, RunSummary};
pub use save::MetaSave;
