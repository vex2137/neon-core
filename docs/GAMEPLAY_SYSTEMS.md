# Gameplay Systems

All original Neon Core gameplay mechanics and balance curves are preserved intact.

## Player & Ships (`src/player/`)

Four distinct playable ship hulls are available in the Hangar (`SHIPS`):
- **INTERCEPTOR** (default, free): Balanced combat baseline.
- **VIPER MK-II** (1,500 Shards): `+22%` move speed, `+28%` fire rate (`0.78x` interval), `+1` starting salvo projectile, `-15%` max HP.
- **TITAN DREADNOUGHT** (4,500 Shards): `+70%` max HP, `+60` base plasma shield, `+35%` damage, 4-shot plasma shotgun (`+3` projectiles, `0.165` spread), `-14%` move speed.
- **PHANTOM WRAITH** (9,000 Shards): `-32%` Dash cooldown, `+15%` base crit chance, drops proximity plasma mines (`4.5x` weapon damage, `138` blast radius) on every Dash.

### Movement & Dash
- Exponential velocity smoothing (`1.0 - (-18.0 * dt).exp()`) clamped inside the circular arena (`ARENA_RADIUS = 2800.0`).
- **Dash** propels the ship at `860` units/sec for `dash_duration` (`0.18s + 0.015s/lvl`) with full invulnerability against enemy contact, enemy projectiles, and orbital strikes.

### Weapons & Passive Abilities
- **Primary Weapon**: Multi-projectile spread salvo with piercing (`pierce`), critical hits (`2x` damage, capped at `95%`), and Overdrive fire-rate (`0.68x` interval) + damage bonuses.
- **Orbital Saws**: Rotating blades at radius `74.0` dealing continuous spatial-grid contact DPS.
- **Chain Tesla**: Periodic lightning arcs jumping across up to `2 + tesla_lvl` nearby enemies within `260` units.
- **EMP Nova**: Radial shockwave pushing and damaging enemies; with **Podwojna Supernova** unlocked, also clears enemy projectiles inside `radius + 45`.

## Enemies & Bosses (`src/enemies/`)

Seven archetypes scale by wave HP (`1 + 0.30*(w-1) + 0.26*(w-4)^1.5`) and speed (`min(+65%)`), multiplied by Difficulty (`NORMAL`, `HARDCORE`, `NIGHTMARE`):
1. **Swarmer** (triangle): Fast melee chaser.
2. **Dasher** (diamond): Alternates slow stalk with `3.25x` lunge bursts.
3. **Tank** (hexagon): High-HP armored bruiser with HP bar.
4. **Sniper** (pentagon): Kites at mid-range, telegraphs a red targeting laser, fires high-speed plasma (3-shot spread if Elite).
5. **Bomber** (green diamond): Rushes the player and detonates into radial shrapnel on proximity or death.
6. **Vortex** (heptagon): Orbits while pulling the player inward (`< 270` units) and emitting spiral volleys.
7. **Overlord Boss** (octagon): Spawns every 3rd wave (scales up to 4 simultaneous bosses on high waves/difficulties). Fires bullet-hell rings and enters an **Enraged** phase below `50%` HP with faster speed, denser rings, and aimed high-velocity shots.

## World Events & Objectives (`src/world/`, `src/progression/`)

- **Wave Cycle (22 seconds per wave)**: Rotates through `Swarm`, `Orbital`, `Sniper`, `Bomber`, and every 3rd wave `Boss`.
- **Orbital Hazard Strikes**: Telegraphed red impact zones with expanding inner rings (Wave 3+ or Wave 2+ on Nightmare).
- **Overload Beacon**: Gold capture zone (`2.4s` hold time) that clears nearby enemy bullets, awards Shards + XP, and activates **Overdrive** (`+30%` move speed, faster fire rate, bonus damage).
- **Combo Chain**: Kills within `3.2s` increment combo, scaling gem XP up to `2.0x`.
- **WASM Demo Mode (`--features demo`)**: Completes with a Victory screen after clearing Wave 8.
