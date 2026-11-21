//! Versioned save serialization (v2 key=value + checksum) and safe migration
//! from legacy v1 comma-separated formats (5, 11, or 21 fields).

use super::save::MetaSave;

pub const SAVE_HEADER_V2: &str = "NEON_CORE_SAVE_V2";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveFormat {
    V1LegacyCsv,
    V2KeyValue,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    CorruptChecksum,
    InvalidLegacyCsv,
    MissingRequiredFields,
}

/// Deterministic 64-bit FNV-1a checksum for save integrity verification.
pub fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// Serializes `MetaSave` to versioned v2 format with trailing integrity checksum.
pub fn serialize_v2(save: &MetaSave) -> String {
    let st = save.settings.validated();
    let body = format!(
        "shards={}\n\
dmg_lvl={}\n\
rate_lvl={}\n\
hp_lvl={}\n\
spd_lvl={}\n\
crit_lvl={}\n\
regen_lvl={}\n\
mag_lvl={}\n\
dash_lvl={}\n\
greed_lvl={}\n\
drone_lvl={}\n\
shield_lvl={}\n\
start_salvo_lvl={}\n\
supernova_lvl={}\n\
overdrive_lvl={}\n\
prestige_lvl={}\n\
selected_ship={}\n\
unlocked_ships_mask={}\n\
difficulty={}\n\
best_wave={}\n\
best_kills={}\n\
set_sound={}\n\
set_master_vol={}\n\
set_sfx_vol={}\n\
set_shake={}\n\
set_hit_stop={}\n\
set_dmg_nums={}\n\
set_high_particles={}\n\
set_fixed_stick={}\n\
set_auto_aim={}\n\
set_auto_fire={}\n\
set_deadzone={}\n\
set_vibration={}\n\
set_perf_hud={}\n",
        save.shards,
        save.dmg_lvl,
        save.rate_lvl,
        save.hp_lvl,
        save.spd_lvl,
        save.crit_lvl,
        save.regen_lvl,
        save.mag_lvl,
        save.dash_lvl,
        save.greed_lvl,
        save.drone_lvl,
        save.shield_lvl,
        save.start_salvo_lvl,
        save.supernova_lvl,
        save.overdrive_lvl,
        save.prestige_lvl,
        save.selected_ship,
        save.unlocked_ships_mask,
        save.difficulty,
        save.best_wave,
        save.best_kills,
        u32::from(st.sound),
        st.master_volume,
        st.sfx_volume,
        st.shake_intensity,
        u32::from(st.hit_stop),
        u32::from(st.dmg_nums),
        u32::from(st.high_particles),
        u32::from(st.fixed_stick),
        u32::from(st.auto_aim),
        u32::from(st.auto_fire),
        st.stick_deadzone,
        u32::from(st.vibration),
        u32::from(st.perf_hud)
    );
    let csum = fnv1a_64(body.as_bytes());
    format!("{SAVE_HEADER_V2}\n{body}checksum={csum:016x}\n")
}

/// Parses either a v2 versioned save or a legacy v1 CSV save, validating and clamping all fields.
pub fn parse_save(raw: &str) -> Result<(MetaSave, SaveFormat), ParseError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(ParseError::Empty);
    }

    if let Some(first_line) = trimmed.lines().next() {
        if first_line.trim() == SAVE_HEADER_V2 {
            let save = parse_v2(trimmed)?;
            return Ok((save.validated(), SaveFormat::V2KeyValue));
        }
    }

    let save = parse_v1_csv(trimmed)?;
    Ok((save.validated(), SaveFormat::V1LegacyCsv))
}

fn parse_v2(raw: &str) -> Result<MetaSave, ParseError> {
    let mut save = MetaSave::default();
    let mut body = String::with_capacity(raw.len());
    let mut found_checksum: Option<u64> = None;
    let mut parsed_keys = 0_usize;

    for line in raw.lines().skip(1) {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let Some((key, val)) = l.split_once('=') else {
            return Err(ParseError::CorruptChecksum);
        };
        let key = key.trim();
        let val = val.trim();
        if key == "checksum" {
            let parsed = u64::from_str_radix(val, 16).map_err(|_| ParseError::CorruptChecksum)?;
            found_checksum = Some(parsed);
            continue;
        }

        body.push_str(key);
        body.push('=');
        body.push_str(val);
        body.push('\n');

        let num: u32 = val.parse().map_err(|_| ParseError::CorruptChecksum)?;
        parsed_keys += 1;
        match key {
            "shards" => save.shards = num,
            "dmg_lvl" => save.dmg_lvl = num,
            "rate_lvl" => save.rate_lvl = num,
            "hp_lvl" => save.hp_lvl = num,
            "spd_lvl" => save.spd_lvl = num,
            "crit_lvl" => save.crit_lvl = num,
            "regen_lvl" => save.regen_lvl = num,
            "mag_lvl" => save.mag_lvl = num,
            "dash_lvl" => save.dash_lvl = num,
            "greed_lvl" => save.greed_lvl = num,
            "drone_lvl" => save.drone_lvl = num,
            "shield_lvl" => save.shield_lvl = num,
            "start_salvo_lvl" => save.start_salvo_lvl = num,
            "supernova_lvl" => save.supernova_lvl = num,
            "overdrive_lvl" => save.overdrive_lvl = num,
            "prestige_lvl" => save.prestige_lvl = num,
            "selected_ship" => save.selected_ship = num,
            "unlocked_ships_mask" => save.unlocked_ships_mask = num,
            "difficulty" => save.difficulty = num,
            "best_wave" => save.best_wave = num,
            "best_kills" => save.best_kills = num,
            "set_sound" => save.settings.sound = num != 0,
            "set_master_vol" => save.settings.master_volume = num.min(100) as u8,
            "set_sfx_vol" => save.settings.sfx_volume = num.min(100) as u8,
            "set_shake" => save.settings.shake_intensity = num.min(100) as u8,
            "set_hit_stop" => save.settings.hit_stop = num != 0,
            "set_dmg_nums" => save.settings.dmg_nums = num != 0,
            "set_high_particles" => save.settings.high_particles = num != 0,
            "set_fixed_stick" => save.settings.fixed_stick = num != 0,
            "set_auto_aim" => save.settings.auto_aim = num != 0,
            "set_auto_fire" => save.settings.auto_fire = num != 0,
            "set_deadzone" => save.settings.stick_deadzone = num.min(30) as u8,
            "set_vibration" => save.settings.vibration = num != 0,
            "set_perf_hud" => save.settings.perf_hud = num != 0,
            _ => {
                // Unknown optional fields from future versions are safely ignored.
            }
        }
    }

    if parsed_keys == 0 {
        return Err(ParseError::MissingRequiredFields);
    }

    if let Some(expected) = found_checksum {
        let actual = fnv1a_64(body.as_bytes());
        if actual != expected {
            return Err(ParseError::CorruptChecksum);
        }
    }

    Ok(save)
}

/// Strictly parses legacy v1 CSV (5, 11, or 21 comma-separated `u32` fields).
/// Unlike the old `filter_map`, malformed middle fields reject the corrupt record
/// instead of silently shifting subsequent upgrade columns left.
fn parse_v1_csv(raw: &str) -> Result<MetaSave, ParseError> {
    let mut parts = Vec::with_capacity(21);
    for tok in raw.split(',') {
        let parsed: u32 = tok
            .trim()
            .parse()
            .map_err(|_| ParseError::InvalidLegacyCsv)?;
        parts.push(parsed);
    }

    if parts.len() < 5 {
        return Err(ParseError::MissingRequiredFields);
    }

    let mut s = MetaSave {
        shards: parts[0],
        dmg_lvl: parts[1],
        rate_lvl: parts[2],
        hp_lvl: parts[3],
        spd_lvl: parts[4],
        ..MetaSave::default()
    };

    if parts.len() >= 11 {
        s.crit_lvl = parts[5];
        s.regen_lvl = parts[6];
        s.mag_lvl = parts[7];
        s.dash_lvl = parts[8];
        s.greed_lvl = parts[9];
        s.drone_lvl = parts[10];
    }

    if parts.len() >= 21 {
        s.shield_lvl = parts[11];
        s.start_salvo_lvl = parts[12];
        s.supernova_lvl = parts[13];
        s.overdrive_lvl = parts[14];
        s.prestige_lvl = parts[15];
        s.selected_ship = parts[16];
        s.unlocked_ships_mask = parts[17];
        s.difficulty = parts[18];
        s.best_wave = parts[19];
        s.best_kills = parts[20];
    }

    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v2_round_trip_preserves_all_fields() {
        let original = MetaSave {
            shards: 12_345,
            dmg_lvl: 10,
            rate_lvl: 8,
            hp_lvl: 9,
            spd_lvl: 7,
            crit_lvl: 6,
            regen_lvl: 5,
            mag_lvl: 4,
            dash_lvl: 3,
            greed_lvl: 8,
            drone_lvl: 4,
            shield_lvl: 6,
            start_salvo_lvl: 3,
            supernova_lvl: 5,
            overdrive_lvl: 4,
            prestige_lvl: 2,
            selected_ship: 2,
            unlocked_ships_mask: 0b0111,
            difficulty: 2,
            best_wave: 24,
            best_kills: 1850,
            settings: crate::game::state::Settings::default(),
        };
        let encoded = serialize_v2(&original);
        let (decoded, fmt) = parse_save(&encoded).expect("v2 save must parse");
        assert_eq!(fmt, SaveFormat::V2KeyValue);
        assert_eq!(decoded, original);
    }

    #[test]
    fn migrates_legacy_v1_5_11_and_21_field_csv() {
        let (s5, fmt5) = parse_save("500,3,2,4,1").expect("5-field v1 must parse");
        assert_eq!(fmt5, SaveFormat::V1LegacyCsv);
        assert_eq!(s5.shards, 500);
        assert_eq!(s5.dmg_lvl, 3);
        assert_eq!(s5.unlocked_ships_mask, 1);

        let (s11, _) = parse_save("900,4,3,2,1,5,6,7,2,3,4").expect("11-field v1 must parse");
        assert_eq!(s11.crit_lvl, 5);
        assert_eq!(s11.drone_lvl, 4);

        let (s21, _) = parse_save("30000,15,12,15,10,8,10,8,6,8,4,10,5,8,8,3,2,7,1,19,999")
            .expect("21-field v1 must parse");
        assert_eq!(s21.shards, 30_000);
        assert_eq!(s21.prestige_lvl, 3);
        assert_eq!(s21.selected_ship, 2);
        assert_eq!(s21.best_wave, 19);
    }

    #[test]
    fn rejects_corrupted_middle_csv_field_instead_of_shifting_indices() {
        let err = parse_save("500,3,CORRUPT,4,1,2,3").unwrap_err();
        assert_eq!(err, ParseError::InvalidLegacyCsv);
    }

    #[test]
    fn rejects_tampered_v2_checksum() {
        let original = MetaSave {
            shards: 100,
            ..MetaSave::default()
        };
        let tampered = serialize_v2(&original).replace("shards=100", "shards=999999");
        assert_eq!(
            parse_save(&tampered).unwrap_err(),
            ParseError::CorruptChecksum
        );
    }

    #[test]
    fn handles_missing_optional_v2_fields_and_clamps_invalid_values() {
        let partial =
            format!("{SAVE_HEADER_V2}\nshards=420\ndmg_lvl=999\nselected_ship=99\ndifficulty=42\n");
        let (loaded, fmt) = parse_save(&partial).expect("partial v2 without checksum must parse");
        assert_eq!(fmt, SaveFormat::V2KeyValue);
        assert_eq!(loaded.shards, 420);
        assert_eq!(loaded.dmg_lvl, 15); // clamped to ShopItem::max_lvl
        assert_eq!(loaded.selected_ship, 0); // invalid ship resets to Interceptor
        assert_eq!(loaded.difficulty, 2); // clamped to NIGHTMARE
        assert_eq!(loaded.unlocked_ships_mask, 1);
    }
}
