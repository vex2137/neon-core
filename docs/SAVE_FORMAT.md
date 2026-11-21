# Save File Format & Migration

Save persistence is implemented in `src/persistence/save.rs` and `src/persistence/migration.rs`.

## File Locations

- **Android**: `/data/data/dev.vex.neon_core/files/save.dat`
- **Desktop**: `neon_core_save.dat` in the working directory
- **WebAssembly**: In-memory session state (no native filesystem)

## Format V2 (Current)

Saves are written in a human-readable, forward-compatible `key=value` text format protected by a 64-bit FNV-1a checksum:

```ini
NEON_CORE_SAVE_V2
shards=12500
dmg_lvl=8
rate_lvl=6
...
best_wave=19
best_kills=1420
checksum=0123456789abcdef
```

- **Unknown keys** from future versions are safely ignored.
- **Missing optional keys** fall back to defaults (`0`, or `unlocked_ships_mask = 1`).
- **Out-of-range values** are clamped via `MetaSave::validated()` (upgrade levels capped at each `ShopItem::max_lvl`, `difficulty` clamped to `0..=2`, `selected_ship` verified against `unlocked_ships_mask`).

## Legacy V1 Migration (Backward Compatibility)

Older releases stored a single comma-separated line of `u32` integers:
- **5 fields**: `shards,dmg,rate,hp,spd`
- **11 fields**: `+ crit,regen,mag,dash,greed,drone`
- **21 fields**: `+ shield,start_salvo,supernova,overdrive,prestige,selected_ship,unlocked_ships_mask,difficulty,best_wave,best_kills`

When a legacy V1 save is detected:
1. Every token is parsed strictly (malformed middle fields are rejected instead of shifting subsequent indices).
2. All fields are validated and clamped.
3. The original valid file is copied to `save.dat.bak` before atomically migrating `save.dat` to V2.

## Corruption Protection & Atomic Writes

`MetaSave::save_to_path` never writes directly over the live save file:
1. Serializes V2 data and writes + `sync_all()`s to `save.dat.tmp`.
2. If an existing `save.dat` parses cleanly, copies it to `save.dat.bak`.
3. Atomically renames `save.dat.tmp` over `save.dat`.

On load (`MetaSave::load_from_path`):
- If `save.dat` fails parsing or checksum validation, its raw bytes are quarantined to `save.dat.corrupt` (if not already present) so player data is never silently destroyed.
- The loader then attempts automatic recovery from `save.dat.bak`.
- Only if both primary and backup are missing or invalid does it fall back to `MetaSave::default()`.
