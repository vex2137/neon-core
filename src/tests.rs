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
