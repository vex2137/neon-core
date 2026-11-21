//! Persistent meta-progression save data with validation, atomic writes, and backup recovery.

#[cfg(any(not(target_arch = "wasm32"), test))]
use super::migration::{parse_save, serialize_v2, SaveFormat};
#[cfg(any(not(target_arch = "wasm32"), test))]
use crate::player::SHIPS;
use crate::progression::max_level_for_id;
#[cfg(any(not(target_arch = "wasm32"), test))]
use crate::progression::DIFFICULTIES;
#[cfg(any(not(target_arch = "wasm32"), test))]
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MetaSave {
    pub shards: u32,
    pub dmg_lvl: u32,
    pub rate_lvl: u32,
    pub hp_lvl: u32,
    pub spd_lvl: u32,
    pub crit_lvl: u32,
    pub regen_lvl: u32,
    pub mag_lvl: u32,
    pub dash_lvl: u32,
    pub greed_lvl: u32,
    pub drone_lvl: u32,
    pub shield_lvl: u32,
    pub start_salvo_lvl: u32,
    pub supernova_lvl: u32,
    pub overdrive_lvl: u32,
    pub prestige_lvl: u32,
    pub selected_ship: u32,
    pub unlocked_ships_mask: u32,
    pub difficulty: u32,
    pub best_wave: u32,
    pub best_kills: u32,
}

impl Default for MetaSave {
    fn default() -> Self {
        Self {
            shards: 0,
            dmg_lvl: 0,
            rate_lvl: 0,
            hp_lvl: 0,
            spd_lvl: 0,
            crit_lvl: 0,
            regen_lvl: 0,
            mag_lvl: 0,
            dash_lvl: 0,
            greed_lvl: 0,
            drone_lvl: 0,
            shield_lvl: 0,
            start_salvo_lvl: 0,
            supernova_lvl: 0,
            overdrive_lvl: 0,
            prestige_lvl: 0,
            selected_ship: 0,
            unlocked_ships_mask: 1,
            difficulty: 0,
            best_wave: 0,
            best_kills: 0,
        }
    }
}

impl MetaSave {
    /// Clamps every field to valid game bounds so corrupted values can never break combat math.
    #[cfg(any(not(target_arch = "wasm32"), test))]
    pub fn validated(mut self) -> Self {
        self.dmg_lvl = self.dmg_lvl.min(max_level_for_id(0));
        self.rate_lvl = self.rate_lvl.min(max_level_for_id(1));
        self.crit_lvl = self.crit_lvl.min(max_level_for_id(2));
        self.drone_lvl = self.drone_lvl.min(max_level_for_id(3));
        self.dash_lvl = self.dash_lvl.min(max_level_for_id(4));
        self.hp_lvl = self.hp_lvl.min(max_level_for_id(5));
        self.regen_lvl = self.regen_lvl.min(max_level_for_id(6));
        self.spd_lvl = self.spd_lvl.min(max_level_for_id(7));
        self.mag_lvl = self.mag_lvl.min(max_level_for_id(8));
        self.greed_lvl = self.greed_lvl.min(max_level_for_id(9));
        self.shield_lvl = self.shield_lvl.min(max_level_for_id(10));
        self.start_salvo_lvl = self.start_salvo_lvl.min(max_level_for_id(11));
        self.supernova_lvl = self.supernova_lvl.min(max_level_for_id(12));
        self.overdrive_lvl = self.overdrive_lvl.min(max_level_for_id(13));
        self.prestige_lvl = self.prestige_lvl.min(max_level_for_id(14));

        let valid_ship_mask = (1_u32 << SHIPS.len()) - 1;
        self.unlocked_ships_mask = (self.unlocked_ships_mask & valid_ship_mask) | 1;
        if (self.selected_ship as usize) >= SHIPS.len()
            || (self.unlocked_ships_mask & (1_u32 << self.selected_ship)) == 0
        {
            self.selected_ship = 0;
        }
        self.difficulty = self.difficulty.min((DIFFICULTIES.len() - 1) as u32);
        self
    }

    #[cfg(any(not(target_arch = "wasm32"), test))]
    pub fn default_save_path() -> PathBuf {
        #[cfg(target_os = "android")]
        {
            let dir = Path::new("/data/data/dev.vex.neon_core/files");
            let _ = std::fs::create_dir_all(dir);
            return dir.join("save.dat");
        }
        #[cfg(not(target_os = "android"))]
        {
            if Path::new("/data/data/dev.vex.neon_core").exists() {
                let dir = Path::new("/data/data/dev.vex.neon_core/files");
                let _ = std::fs::create_dir_all(dir);
                dir.join("save.dat")
            } else {
                PathBuf::from("neon_core_save.dat")
            }
        }
    }

    pub fn load() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self::default()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::load_from_path(&Self::default_save_path())
        }
    }

    pub fn save(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = self.save_to_path(&Self::default_save_path());
        }
    }

    /// Loads from `path`, falling back to `<path>.bak` if primary is corrupted.
    /// If the primary file is corrupted, it is preserved as `<path>.corrupt` instead of being silently destroyed.
    #[cfg(any(not(target_arch = "wasm32"), test))]
    pub fn load_from_path(path: &Path) -> Self {
        if let Ok(raw) = std::fs::read_to_string(path) {
            match parse_save(&raw) {
                Ok((loaded, fmt)) => {
                    if fmt == SaveFormat::V1LegacyCsv {
                        // Migrate cleanly to v2 while preserving legacy backup first.
                        let _ = loaded.save_to_path(path);
                    }
                    return loaded;
                }
                Err(_) => {
                    // Quarantine corrupt primary file without losing its raw bytes.
                    let corrupt_path = with_suffix(path, ".corrupt");
                    if !corrupt_path.exists() {
                        let _ = std::fs::copy(path, &corrupt_path);
                    }
                }
            }
        }

        // Attempt recovery from the last known good backup.
        let bak_path = with_suffix(path, ".bak");
        if let Ok(raw_bak) = std::fs::read_to_string(&bak_path) {
            if let Ok((recovered, _)) = parse_save(&raw_bak) {
                let _ = recovered.save_to_path(path);
                return recovered;
            }
        }

        Self::default()
    }

    /// Writes atomically via `<path>.tmp` -> `<path>` while keeping the previous valid save in `<path>.bak`.
    #[cfg(any(not(target_arch = "wasm32"), test))]
    pub fn save_to_path(&self, path: &Path) -> std::io::Result<()> {
        let valid = self.validated();
        let encoded = serialize_v2(&valid);

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let tmp_path = with_suffix(path, ".tmp");
        let bak_path = with_suffix(path, ".bak");

        // 1. Write and flush new save to temporary file first.
        {
            use std::io::Write;
            let mut file = std::fs::File::create(&tmp_path)?;
            file.write_all(encoded.as_bytes())?;
            file.sync_all()?;
        }

        // 2. Preserve existing valid save as `.bak` before replacing primary.
        if path.exists() {
            if let Ok(prev) = std::fs::read_to_string(path) {
                if parse_save(&prev).is_ok() {
                    let _ = std::fs::copy(path, &bak_path);
                }
            }
        }

        // 3. Atomically replace primary file with the verified temp file.
        std::fs::rename(&tmp_path, path)?;
        Ok(())
    }

    pub fn get_lvl(&self, id: u8) -> u32 {
        match id {
            0 => self.dmg_lvl,
            1 => self.rate_lvl,
            2 => self.crit_lvl,
            3 => self.drone_lvl,
            4 => self.dash_lvl,
            5 => self.hp_lvl,
            6 => self.regen_lvl,
            7 => self.spd_lvl,
            8 => self.mag_lvl,
            9 => self.greed_lvl,
            10 => self.shield_lvl,
            11 => self.start_salvo_lvl,
            12 => self.supernova_lvl,
            13 => self.overdrive_lvl,
            _ => self.prestige_lvl,
        }
    }

    pub fn inc_lvl(&mut self, id: u8) {
        let cap = max_level_for_id(id);
        let slot = match id {
            0 => &mut self.dmg_lvl,
            1 => &mut self.rate_lvl,
            2 => &mut self.crit_lvl,
            3 => &mut self.drone_lvl,
            4 => &mut self.dash_lvl,
            5 => &mut self.hp_lvl,
            6 => &mut self.regen_lvl,
            7 => &mut self.spd_lvl,
            8 => &mut self.mag_lvl,
            9 => &mut self.greed_lvl,
            10 => &mut self.shield_lvl,
            11 => &mut self.start_salvo_lvl,
            12 => &mut self.supernova_lvl,
            13 => &mut self.overdrive_lvl,
            _ => &mut self.prestige_lvl,
        };
        *slot = slot.saturating_add(1).min(cap);
    }
}

#[cfg(any(not(target_arch = "wasm32"), test))]
fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(suffix);
    PathBuf::from(os)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_save_path(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("neon_core_test_{}_{}", std::process::id(), name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create test temp dir");
        dir.join("save.dat")
    }

    #[test]
    fn atomic_save_and_backup_recovery_survives_corrupted_primary() {
        let path = temp_save_path("backup_recovery");
        let first = MetaSave {
            shards: 2500,
            dmg_lvl: 5,
            best_wave: 12,
            ..MetaSave::default()
        };
        first.save_to_path(&path).expect("first save");

        let second = MetaSave {
            shards: 4200,
            dmg_lvl: 6,
            best_wave: 15,
            ..first
        };
        second
            .save_to_path(&path)
            .expect("second save creates .bak");
        assert!(with_suffix(&path, ".bak").exists());

        // Corrupt primary save file.
        std::fs::write(&path, "CORRUPTED_GARBAGE!!!").expect("write corrupt data");

        let recovered = MetaSave::load_from_path(&path);
        assert_eq!(recovered.shards, 2500);
        assert_eq!(recovered.best_wave, 12);
        assert!(with_suffix(&path, ".corrupt").exists());

        let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
    }

    #[test]
    fn legacy_v1_file_migrates_cleanly_on_load_with_backup() {
        let path = temp_save_path("legacy_migration");
        std::fs::write(&path, "1200,3,2,4,1,2,1,2,1,3,1").expect("write v1 save");

        let loaded = MetaSave::load_from_path(&path);
        assert_eq!(loaded.shards, 1200);
        assert_eq!(loaded.dmg_lvl, 3);
        assert_eq!(loaded.drone_lvl, 1);

        // Primary is now v2, and original v1 was preserved in .bak.
        let disk = std::fs::read_to_string(&path).expect("read migrated");
        assert!(disk.starts_with(super::super::migration::SAVE_HEADER_V2));
        assert!(with_suffix(&path, ".bak").exists());

        let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
    }

    #[test]
    fn missing_file_returns_default_configuration() {
        let path = temp_save_path("missing");
        let loaded = MetaSave::load_from_path(&path);
        assert_eq!(loaded, MetaSave::default());
        let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
    }
}
