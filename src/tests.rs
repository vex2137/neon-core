//! Integration, regression, determinism, and simulation performance benchmark tests.

use crate::combat::Bullet;
use crate::enemies::EnemyKind;
use crate::game::{Game, ScreenState, Viewport};
use crate::input::FrameInput;
use crate::persistence::MetaSave;
use crate::progression::WAVE_DURATION;
use macroquad::math::{vec2, Vec2};

#[test]
fn full_simulation_is_deterministic_for_identical_seeds_and_inputs() {
    let vp = Viewport::from_dimensions(2400.0, 1080.0);
    let mut a = Game::new_headless(2026, MetaSave::default());
    let mut b = Game::new_headless(2026, MetaSave::default());
    a.start_run(vp);
    b.start_run(vp);

    let input = FrameInput {
        move_dir: vec2(0.8, -0.6).normalize(),
        manual_aim_dir: None,
        fire_held: false,
        dash_pressed: false,
        pause_pressed: false,
    };

    for step in 0..180 {
        let mut frame_in = input;
        if step == 45 {
            frame_in.dash_pressed = true;
        }
        a.step_playing(1.0 / 60.0, frame_in, vp);
        b.step_playing(1.0 / 60.0, frame_in, vp);
        if a.state == ScreenState::LevelUp {
            let pick = a.level_choices[0].id;
            a.apply_upgrade(pick);
            b.apply_upgrade(pick);
        }
    }

    assert_eq!(a.player.pos, b.player.pos);
    assert_eq!(a.kills, b.kills);
    assert_eq!(a.enemies.len(), b.enemies.len());
    assert_eq!(a.run_shards, b.run_shards);
    assert_eq!(a.level, b.level);
}

#[test]
fn dead_enemies_do_not_take_damage_or_award_double_kills() {
    let vp = Viewport::from_dimensions(1280.0, 720.0);
    let mut game = Game::new_headless(77, MetaSave::default());
    game.start_run(vp);
    game.enemies.clear();
    game.spawn_enemy_at(EnemyKind::Swarmer, vec2(80.0, 0.0), false);
    assert_eq!(game.enemies.len(), 1);

    game.damage_enemy(0, 500.0, vec2(80.0, 0.0), false);
    assert!(!game.enemies[0].alive);
    assert_eq!(game.kills, 1);

    // Calling damage_enemy again on the dead index must be a complete no-op.
    game.damage_enemy(0, 500.0, vec2(80.0, 0.0), false);
    assert_eq!(game.kills, 1);
    assert_eq!(game.gems.len(), 1);
}

#[test]
fn bullet_collision_kills_enemy_and_consumes_pierce() {
    let vp = Viewport::from_dimensions(1280.0, 720.0);
    let mut game = Game::new_headless(99, MetaSave::default());
    game.start_run(vp);
    game.enemies.clear();
    game.player.fire_cd = 1.0;
    game.spawn_enemy_at(EnemyKind::Swarmer, vec2(100.0, 0.0), false);
    game.bullets.push(Bullet {
        pos: vec2(96.0, 0.0),
        vel: vec2(400.0, 0.0),
        dmg: 100.0,
        life: 1.0,
        pierce: 1,
    });

    game.step_playing(1.0 / 60.0, FrameInput::default(), vp);
    assert_eq!(game.kills, 1);
    assert!(game.bullets.is_empty());
    assert_eq!(game.gems.len(), 1);
}

#[test]
fn wave_progression_advances_after_wave_duration_and_spawns_boss_on_wave_3() {
    let vp = Viewport::from_dimensions(1280.0, 720.0);
    let mut game = Game::new_headless(333, MetaSave::default());
    game.start_run(vp);
    assert_eq!(game.wave, 1);

    game.wave_timer = WAVE_DURATION - 0.01;
    game.step_playing(0.02, FrameInput::default(), vp);
    assert_eq!(game.wave, 2);

    game.wave_timer = WAVE_DURATION - 0.01;
    game.step_playing(0.02, FrameInput::default(), vp);
    assert_eq!(game.wave, 3);
    assert!(game.enemies.iter().any(|e| e.kind == EnemyKind::Boss));
}

#[test]
fn dash_grants_invulnerability_against_enemy_bullets_and_hazards() {
    let vp = Viewport::from_dimensions(1280.0, 720.0);
    let mut game = Game::new_headless(55, MetaSave::default());
    game.start_run(vp);
    game.enemies.clear();
    let start_hp = game.player.hp;

    // Trigger dash on this frame while an enemy bullet sits directly on top of player.
    game.spawn_e_bullet(Vec2::ZERO, Vec2::ZERO, 50.0, 12.0);
    let input = FrameInput {
        move_dir: vec2(1.0, 0.0),
        dash_pressed: true,
        ..FrameInput::default()
    };
    game.step_playing(1.0 / 60.0, input, vp);
    assert_eq!(game.player.hp, start_hp);
    assert!(game.player.dash_timer > 0.0);
    assert!(game.player.dash_cd > 0.0);
}

#[test]
fn level_up_clears_active_touch_sticks_regression() {
    let vp = Viewport::from_dimensions(1280.0, 720.0);
    let mut game = Game::new_headless(88, MetaSave::default());
    game.start_run(vp);
    game.touch.left_stick.touch_id = Some(42);
    game.touch.left_stick.dir = vec2(1.0, 0.0);

    game.roll_levelup();
    assert_eq!(game.state, ScreenState::LevelUp);
    assert_eq!(game.touch.left_stick.touch_id, None);
    assert_eq!(game.touch.left_stick.dir, Vec2::ZERO);
}

#[test]
fn chain_tesla_and_emp_nova_damage_nearby_enemies_without_panicking() {
    let vp = Viewport::from_dimensions(1280.0, 720.0);
    let mut game = Game::new_headless(1234, MetaSave::default());
    game.start_run(vp);
    game.enemies.clear();
    game.player.tesla_lvl = 3;
    game.player.nova_lvl = 2;

    for i in 0..8 {
        let a = i as f32 * std::f32::consts::TAU / 8.0;
        game.spawn_enemy_at(EnemyKind::Tank, Vec2::from_angle(a) * 110.0, false);
    }
    game.rebuild_spatial_grid();

    let hp_before: f32 = game.enemies.iter().map(|e| e.hp).sum();
    game.trigger_tesla();
    game.trigger_nova();
    let hp_after: f32 = game.enemies.iter().map(|e| e.hp).sum();
    assert!(hp_after < hp_before);
    assert!(!game.effects.arcs.is_empty());
}

#[test]
fn benchmark_heavy_combat_simulation_step() {
    let vp = Viewport::from_dimensions(2400.0, 1080.0);
    let meta = MetaSave {
        drone_lvl: 4,
        supernova_lvl: 6,
        start_salvo_lvl: 4,
        shield_lvl: 5,
        ..MetaSave::default()
    };
    let mut game = Game::new_headless(202601, meta);
    game.start_run(vp);

    // Populate heavy combat scenario: 300 enemies, 180 bullets, 120 enemy bullets.
    for i in 0..300 {
        let a = i as f32 * 0.21;
        let r = 180.0 + (i % 15) as f32 * 45.0;
        let kind = EnemyKind::from_u8((i % 7) as u8);
        game.spawn_enemy_at(kind, Vec2::from_angle(a) * r, true);
    }
    for i in 0..120 {
        let a = i as f32 * 0.3;
        game.spawn_e_bullet(
            Vec2::from_angle(a) * 400.0,
            Vec2::from_angle(a + 2.0) * 200.0,
            10.0,
            5.0,
        );
    }

    let input = FrameInput {
        move_dir: vec2(0.7, 0.7).normalize(),
        ..FrameInput::default()
    };
    let frames = 240;
    let t0 = std::time::Instant::now();
    for _ in 0..frames {
        game.player.hp = game.player.max_hp; // keep alive for full benchmark
        game.step_playing(1.0 / 60.0, input, vp);
        if game.state == ScreenState::LevelUp {
            game.apply_upgrade(game.level_choices[0].id);
        }
    }
    let elapsed = t0.elapsed();
    let avg_us = elapsed.as_secs_f64() * 1_000_000.0 / frames as f64;
    println!(
        "[BENCH] heavy_combat_300_enemies: {} frames in {:.2} ms (avg {:.1} us/frame, kills={}, grid_cells={})",
        frames,
        elapsed.as_secs_f64() * 1000.0,
        avg_us,
        game.kills,
        game.grid.occupied_cells()
    );
    // Sanity check: simulation stays well under a 16.6ms 60fps budget even in debug mode.
    assert!(avg_us < 16_000.0);
}

#[test]
fn benchmark_textbuf_vs_heap_string_and_spatial_grid_vs_linear() {
    use crate::effects::TextBuf;
    use crate::rng::GameRng;
    use crate::world::SpatialGrid;
    use std::fmt::Write;
    use std::hint::black_box;

    // 1. Hot-path formatting: heap `format!` (before) vs stack `TextBuf` (after)
    let iters = 50_000_usize;
    let t_heap = std::time::Instant::now();
    let mut heap_bytes = 0_usize;
    for i in 0..iters {
        let s = format!("HULL {}/{} | {}!", i % 300, 300, i % 999);
        heap_bytes += black_box(s.len());
    }
    let heap_us = t_heap.elapsed().as_secs_f64() * 1_000_000.0;

    let t_stack = std::time::Instant::now();
    let mut stack_bytes = 0_usize;
    for i in 0..iters {
        let mut tb = TextBuf::<32>::new();
        let _ = write!(&mut tb, "HULL {}/{} | {}!", i % 300, 300, i % 999);
        stack_bytes += black_box(tb.as_str().len());
    }
    let stack_us = t_stack.elapsed().as_secs_f64() * 1_000_000.0;
    assert_eq!(heap_bytes, stack_bytes);

    // 2. Local bullet-vs-enemy queries: linear O(N) scan vs SpatialGrid O(1) cell lookup
    let mut rng = GameRng::seeded(555);
    let enemies: Vec<Vec2> = (0..400)
        .map(|_| {
            vec2(
                rng.range_f32(-2600.0, 2600.0),
                rng.range_f32(-2600.0, 2600.0),
            )
        })
        .collect();
    let queries: Vec<Vec2> = (0..200)
        .map(|_| {
            vec2(
                rng.range_f32(-2600.0, 2600.0),
                rng.range_f32(-2600.0, 2600.0),
            )
        })
        .collect();
    let mut grid = SpatialGrid::with_capacity(420);
    grid.rebuild(enemies.iter().enumerate().map(|(i, &p)| (i, true, p)));

    let rounds = 200;
    let t_lin = std::time::Instant::now();
    let mut lin_hits = 0_usize;
    for _ in 0..rounds {
        for &q in &queries {
            for &ep in &enemies {
                if ep.distance_squared(q) < 45.0 * 45.0 {
                    lin_hits += 1;
                }
            }
        }
    }
    let lin_us = t_lin.elapsed().as_secs_f64() * 1_000_000.0;

    let t_grid = std::time::Instant::now();
    let mut grid_hits = 0_usize;
    for _ in 0..rounds {
        for &q in &queries {
            grid.for_each_in_aabb(q, 45.0, |idx| {
                if enemies[idx].distance_squared(q) < 45.0 * 45.0 {
                    grid_hits += 1;
                }
                true
            });
        }
    }
    let grid_us = t_grid.elapsed().as_secs_f64() * 1_000_000.0;
    assert_eq!(lin_hits, grid_hits);

    println!(
        "[BENCH] formatting ({} iters): heap String={:.1} us vs stack TextBuf={:.1} us ({:.2}x faster)",
        iters,
        heap_us,
        stack_us,
        heap_us / stack_us.max(1.0)
    );
    println!(
        "[BENCH] 400-enemy x 200-bullet queries ({} rounds): linear={:.1} us vs SpatialGrid={:.1} us ({:.2}x faster, hits={})",
        rounds,
        lin_us,
        grid_us,
        lin_us / grid_us.max(1.0),
        grid_hits
    );
}

#[test]
fn mid_run_save_and_resume_preserves_full_state_and_determinism() {
    use crate::persistence::run_save::{delete_run_save_at, RunSnapshot};
    use crate::world::hazards::spawn_hazard;

    let vp = Viewport::from_dimensions(2400.0, 1080.0);
    let meta = MetaSave {
        selected_ship: 3, // Phantom Wraith
        unlocked_ships_mask: 0b1111,
        difficulty: 1, // Hardcore
        shield_lvl: 3,
        ..MetaSave::default()
    };
    let mut original = Game::new_headless(424242, meta);
    original.start_run(vp);
    original.wave = 4;
    original.wave_timer = 11.5;
    original.level = 5;
    original.xp = 42.25;
    original.next_xp = 110.0;
    original.combo = 9;
    original.combo_timer = 2.4;
    original.run_shards = 135;
    original.kills = 47;
    original.player.pos = vec2(220.0, -145.0);
    original.player.vel = vec2(60.0, -30.0);
    original.player.hp = 82.0;
    original.player.shield = 44.0;
    original.player.apply_upgrade(4); // Orbital Saws
    original.player.apply_upgrade(5); // Chain Tesla
    original.player.apply_upgrade(6); // EMP Nova
    original.spawn_mine(vec2(180.0, -100.0), 90.0, 138.0);
    spawn_hazard(&mut original.hazards, vec2(260.0, -90.0), 70.0, 1.1, 30.0);
    original
        .beacon
        .spawn_near(original.player.pos, &mut original.rng);
    original.spawn_enemy_at(EnemyKind::Boss, vec2(450.0, -140.0), false);

    let dir = std::env::temp_dir().join(format!("neon_core_run_rt_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("neon_core_run.dat");

    let snap = original.capture_run_snapshot();
    snap.save_to_path(&path).expect("save mid-run snapshot");

    let loaded = RunSnapshot::load_from_path(&path).expect("load mid-run snapshot");
    let mut resumed = Game::new_headless(1, MetaSave::default());
    resumed.restore_run_snapshot(loaded);

    assert_eq!(resumed.meta.selected_ship, 3);
    assert_eq!(resumed.meta.difficulty, 1);
    assert_eq!(resumed.wave, 4);
    assert_eq!(resumed.level, 5);
    assert_eq!(resumed.run_shards, 135);
    assert_eq!(resumed.kills, 47);
    assert_eq!(resumed.player.orbit_count, original.player.orbit_count);
    assert_eq!(resumed.player.tesla_lvl, original.player.tesla_lvl);
    assert_eq!(resumed.player.nova_lvl, original.player.nova_lvl);
    assert_eq!(resumed.mines.len(), original.mines.len());
    assert_eq!(resumed.hazards.len(), original.hazards.len());
    assert_eq!(resumed.beacon.active, original.beacon.active);
    assert_eq!(resumed.enemies.len(), original.enemies.len());
    assert_eq!(resumed.rng.state(), original.rng.state());

    // Step both simulations forward identically — RNG and physics must stay synchronized.
    let frame_in = FrameInput {
        move_dir: vec2(1.0, 0.25).normalize(),
        ..FrameInput::default()
    };
    for _ in 0..30 {
        original.step_playing(1.0 / 60.0, frame_in, vp);
        resumed.step_playing(1.0 / 60.0, frame_in, vp);
    }
    assert!((original.player.pos - resumed.player.pos).length() < 1e-3);
    assert_eq!(original.kills, resumed.kills);
    assert_eq!(original.rng.state(), resumed.rng.state());

    delete_run_save_at(&path);
    assert!(!path.exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn mid_run_save_corruption_recovers_from_backup_and_death_clears_save() {
    use crate::persistence::run_save::{
        delete_run_save_at, parse_run_v1, serialize_run_v1, RunParseError, RunSnapshot,
    };

    let vp = Viewport::from_dimensions(1280.0, 720.0);
    let mut game = Game::new_headless(9001, MetaSave::default());
    game.start_run(vp);
    game.wave = 3;
    game.kills = 25;

    let dir = std::env::temp_dir().join(format!("neon_core_run_bak_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("neon_core_run.dat");

    let snap1 = game.capture_run_snapshot();
    snap1.save_to_path(&path).expect("first run save");

    game.wave = 5;
    game.kills = 60;
    let snap2 = game.capture_run_snapshot();
    snap2
        .save_to_path(&path)
        .expect("second run save creates .bak");

    // Tampered checksum must be rejected directly by parser.
    let tampered = serialize_run_v1(&snap2).replace("wave=5", "wave=99");
    assert_eq!(
        parse_run_v1(&tampered).unwrap_err(),
        RunParseError::CorruptChecksum
    );

    // Corrupt primary on disk -> loader quarantines `.corrupt` and recovers `.bak` (wave 3).
    std::fs::write(&path, tampered).expect("write tampered primary");
    let recovered = RunSnapshot::load_from_path(&path).expect("recover from .bak");
    assert_eq!(recovered.wave, 3);
    assert_eq!(recovered.kills, 25);
    assert!(dir.join("neon_core_run.dat.corrupt").exists());

    // Delete on run completion clears both primary and backup files.
    delete_run_save_at(&path);
    assert!(!path.exists());
    assert!(!dir.join("neon_core_run.dat.bak").exists());

    // In-game finish_run clears cached summary to prevent save-scumming.
    game.save_active_run();
    assert!(game.saved_run_summary.is_some());
    game.finish_run(false);
    assert!(game.saved_run_summary.is_none());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn expanded_settings_persist_in_v2_and_older_v2_saves_keep_defaults() {
    use crate::game::Settings;
    use crate::persistence::migration::{parse_save, serialize_v2, SAVE_HEADER_V2};

    let custom = Settings {
        sound: true,
        master_volume: 70,
        sfx_volume: 60,
        shake_intensity: 50,
        hit_stop: false,
        dmg_nums: false,
        high_particles: false,
        fixed_stick: true,
        auto_aim: false,
        auto_fire: false,
        stick_deadzone: 25,
        vibration: false,
        perf_hud: true,
    };
    let meta = MetaSave {
        shards: 5555,
        dmg_lvl: 7,
        settings: custom,
        ..MetaSave::default()
    };
    let encoded = serialize_v2(&meta);
    let (decoded, _) = parse_save(&encoded).expect("v2 with settings parses");
    assert_eq!(decoded.settings, custom);
    assert!((decoded.settings.effective_sfx_gain() - 0.42).abs() < 1e-4);
    assert!((decoded.settings.shake_scale() - 0.5).abs() < 1e-4);

    // Older v2 save without any `set_*` keys must preserve progression and apply default settings.
    let legacy_v2 = format!("{SAVE_HEADER_V2}\nshards=888\ndmg_lvl=4\nbest_wave=14\n");
    let (loaded, _) = parse_save(&legacy_v2).expect("older v2 without settings parses");
    assert_eq!(loaded.shards, 888);
    assert_eq!(loaded.dmg_lvl, 4);
    assert_eq!(loaded.best_wave, 14);
    assert_eq!(loaded.settings, Settings::default());
}

#[test]
fn settings_affect_auto_fire_shake_hit_stop_and_particle_density() {
    use crate::game::Settings;

    let vp = Viewport::from_dimensions(1280.0, 720.0);
    let meta = MetaSave {
        settings: Settings {
            auto_fire: false,
            shake_intensity: 50,
            hit_stop: false,
            high_particles: false,
            ..Settings::default()
        },
        ..MetaSave::default()
    };
    let mut game = Game::new_headless(707, meta);
    game.start_run(vp);
    game.enemies.clear();
    game.spawn_enemy_at(EnemyKind::Tank, vec2(120.0, 0.0), false);

    // With Auto-Fire OFF and fire_held=false, player must NOT shoot automatically.
    game.step_playing(1.0 / 60.0, FrameInput::default(), vp);
    assert!(game.bullets.is_empty());

    // Holding fire trigger shoots immediately.
    game.step_playing(
        1.0 / 60.0,
        FrameInput {
            fire_held: true,
            ..FrameInput::default()
        },
        vp,
    );
    assert!(!game.bullets.is_empty());

    // Particle cap is 50% on Low density, shake scales by 50%, hit-stop stays 0 when disabled.
    assert_eq!(
        game.effects.max_particles,
        crate::game::config::MAX_PARTICLES / 2
    );
    game.effects.add_shake(10.0, game.settings.shake_scale());
    assert!((game.effects.shake - 5.0).abs() < 1e-4);
    game.effects.add_hit_stop(0.05, game.settings.hit_stop);
    assert_eq!(game.effects.hit_stop, 0.0);
}

#[test]
fn gamepad_deadzone_twin_stick_and_input_merging_work_correctly() {
    use crate::game::Settings;
    use crate::input::{merge_playing_inputs, GamepadManager, RawGamepadState};

    let mut mgr = GamepadManager::headless();
    let settings = Settings {
        stick_deadzone: 20,
        ..Settings::default()
    };

    // Frame 1: drift inside 20% deadzone is ignored; right stick deflected; RT pulled.
    let raw1 = RawGamepadState {
        connected: true,
        left_stick: vec2(0.12, -0.08),
        right_stick: vec2(0.0, -0.9),
        rt: 0.85,
        ..RawGamepadState::default()
    };
    let snap1 = mgr.process_raw(raw1, 1.0 / 60.0, settings);
    assert_eq!(snap1.move_dir, Vec2::ZERO);
    assert!(snap1.aim_dir.is_some());
    assert!((snap1.aim_dir.unwrap() - vec2(0.0, -1.0)).length() < 1e-4);
    assert!(snap1.fire_held);
    assert!(snap1.dash_pressed); // rising edge

    // Frame 2: holding RT keeps fire_held true, but dash_pressed must NOT re-trigger.
    let snap2 = mgr.process_raw(raw1, 1.0 / 60.0, settings);
    assert!(snap2.fire_held);
    assert!(!snap2.dash_pressed);

    // Twin-stick right stick overrides mouse/touch aim in merge_playing_inputs.
    let merged = merge_playing_inputs(
        Vec2::ZERO,
        vec2(1.0, 0.0),
        snap1,
        Some(vec2(1.0, 0.0)),
        Some(vec2(-1.0, 0.0)),
        false,
        false,
        false,
    );
    assert!((merged.move_dir - vec2(1.0, 0.0)).length() < 1e-4);
    assert!((merged.manual_aim_dir.unwrap() - vec2(0.0, -1.0)).length() < 1e-4);
    assert!(merged.fire_held);
    assert!(merged.dash_pressed);
}

#[test]
fn gamepad_menu_levelup_pause_and_settings_navigation_work() {
    use crate::game::Settings;
    use crate::input::MenuNavInput;
    use crate::ui::menus::step_menu_focus;
    use crate::ui::overlays::{step_levelup_nav, step_pause_nav, step_settings_nav, PauseAction};

    // 1. Main menu tab switch (RB) + jump to Resume Run (Back) + Confirm (A)
    let mut tab = 0_u8;
    let mut menu_idx = 0_usize;
    step_menu_focus(
        &mut tab,
        &mut menu_idx,
        MenuNavInput {
            next_tab: true,
            ..MenuNavInput::default()
        },
        true,
    );
    assert_eq!(tab, 1);
    assert_eq!(menu_idx, 0);

    step_menu_focus(
        &mut tab,
        &mut menu_idx,
        MenuNavInput {
            back: true,
            ..MenuNavInput::default()
        },
        true,
    );
    // Tab 1 has 5 cards + 3 difficulties => action_start = 8 (Resume Run)
    assert_eq!(menu_idx, 8);
    let act = step_menu_focus(
        &mut tab,
        &mut menu_idx,
        MenuNavInput {
            confirm: true,
            ..MenuNavInput::default()
        },
        true,
    );
    assert_eq!(act, Some(8));

    // 2. Level-up 3-card draft cycling with RB / Right and confirm
    let mut lu_idx = 0_usize;
    assert_eq!(
        step_levelup_nav(
            &mut lu_idx,
            MenuNavInput {
                next_tab: true,
                ..MenuNavInput::default()
            }
        ),
        None
    );
    assert_eq!(lu_idx, 1);
    assert_eq!(
        step_levelup_nav(
            &mut lu_idx,
            MenuNavInput {
                confirm: true,
                ..MenuNavInput::default()
            }
        ),
        Some(1)
    );

    // 3. Pause overlay navigation to Save & Quit (index 2)
    let mut p_idx = 0_usize;
    step_pause_nav(
        &mut p_idx,
        MenuNavInput {
            down: true,
            ..MenuNavInput::default()
        },
    );
    step_pause_nav(
        &mut p_idx,
        MenuNavInput {
            down: true,
            ..MenuNavInput::default()
        },
    );
    assert_eq!(
        step_pause_nav(
            &mut p_idx,
            MenuNavInput {
                confirm: true,
                ..MenuNavInput::default()
            }
        ),
        Some(PauseAction::SaveAndQuit)
    );

    // 4. Settings overlay slider adjustment (Master Volume -10%) + Back
    let mut st = Settings::default();
    let mut s_idx = 0_usize; // Master Volume slider
    assert!(!step_settings_nav(
        &mut st,
        &mut s_idx,
        MenuNavInput {
            left: true,
            ..MenuNavInput::default()
        }
    ));
    assert_eq!(st.master_volume, 90);
    assert!(step_settings_nav(
        &mut st,
        &mut s_idx,
        MenuNavInput {
            back: true,
            ..MenuNavInput::default()
        }
    ));
}
