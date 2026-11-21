//! Player active/passive abilities: Dash trigger & cooldown helpers.

use super::stats::{ship_by_id, PlayerState};
use crate::audio::SoundBank;
use crate::effects::EffectsPool;
use macroquad::math::Vec2;

/// Attempts to trigger Dash. Returns `Some((pos, dmg, radius))` if Phantom Wraith should drop a mine.
pub fn try_trigger_dash(
    player: &mut PlayerState,
    move_dir: Vec2,
    selected_ship: u32,
    sounds: &SoundBank,
    effects: &mut EffectsPool,
    sound_enabled: bool,
    shake_enabled: bool,
) -> Option<(Vec2, f32, f32)> {
    if player.dash_cd > 0.0 {
        return None;
    }

    sounds.play(&sounds.dash, sound_enabled, 0.8);
    let mine_drop = if selected_ship == 3 {
        Some((player.pos, player.bullet_dmg * 4.5, 138.0))
    } else {
        None
    };

    if move_dir.length_squared() > 0.01 {
        player.dash_dir = move_dir.normalize();
    } else {
        player.dash_dir = Vec2::from_angle(player.aim_angle);
    }

    player.dash_timer = player.dash_duration;
    player.dash_cd = player.dash_max_cd;
    effects.add_shake(2.6, shake_enabled);
    effects.spawn_ring(player.pos, 75.0, ship_by_id(selected_ship).color(), 0.25);

    mine_drop
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::math::vec2;

    #[test]
    fn dash_sets_cooldown_direction_and_drops_phantom_mine() {
        let mut p = PlayerState::default();
        let sounds = SoundBank::empty();
        let mut fx = EffectsPool::new();

        let mine = try_trigger_dash(&mut p, vec2(0.0, -1.0), 3, &sounds, &mut fx, false, true);
        assert!(mine.is_some());
        assert!((p.dash_timer - p.dash_duration).abs() < 1e-5);
        assert!((p.dash_cd - p.dash_max_cd).abs() < 1e-5);
        assert!((p.dash_dir - vec2(0.0, -1.0)).length() < 1e-5);

        // Second immediate call must be rejected by cooldown
        let second = try_trigger_dash(&mut p, vec2(1.0, 0.0), 3, &sounds, &mut fx, false, true);
        assert!(second.is_none());
    }
}
