//! Pure combat damage calculations, critical hits, shield absorption, and kill drops.

use crate::enemies::EnemyKind;
use crate::player::PlayerState;
use crate::rng::GameRng;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitRoll {
    pub damage: f32,
    pub is_crit: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerDamageOutcome {
    pub shield_absorbed: f32,
    pub hp_lost: f32,
    pub lethal: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KillReward {
    pub xp: f32,
    pub shard: u32,
}

/// Resolves a single damage instance with optional 2x critical strike roll.
pub fn roll_hit_damage(
    raw_dmg: f32,
    crit_chance: f32,
    allow_crit: bool,
    rng: &mut GameRng,
) -> HitRoll {
    if !raw_dmg.is_finite() || raw_dmg <= 0.0 {
        return HitRoll {
            damage: 0.0,
            is_crit: false,
        };
    }
    let clamped_crit = if crit_chance.is_finite() {
        crit_chance.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let is_crit = allow_crit && clamped_crit > 0.0 && rng.next_f32() < clamped_crit;
    let damage = if is_crit { raw_dmg * 2.0 } else { raw_dmg };
    HitRoll { damage, is_crit }
}

/// Applies incoming damage to the player's plasma shield first, then hull HP.
pub fn apply_player_damage(player: &mut PlayerState, amount: f32) -> PlayerDamageOutcome {
    if !amount.is_finite() || amount <= 0.0 {
        return PlayerDamageOutcome {
            shield_absorbed: 0.0,
            hp_lost: 0.0,
            lethal: false,
        };
    }

    let mut rem = amount;
    let mut shield_absorbed = 0.0_f32;
    if player.max_shield > 0.0 {
        player.shield_delay = 3.8;
        if player.shield > 0.0 {
            shield_absorbed = player.shield.min(rem);
            player.shield -= shield_absorbed;
            rem -= shield_absorbed;
        }
    }

    if rem <= 0.0 {
        return PlayerDamageOutcome {
            shield_absorbed,
            hp_lost: 0.0,
            lethal: false,
        };
    }

    player.hp -= rem;
    let lethal = player.hp <= 0.0;
    if lethal {
        player.hp = 0.0;
    }

    PlayerDamageOutcome {
        shield_absorbed,
        hp_lost: rem,
        lethal,
    }
}

/// Computes gem XP and shard drops when an enemy is destroyed.
pub fn roll_kill_reward(kind: EnemyKind, elite: bool, rng: &mut GameRng) -> KillReward {
    let mut shard: u32 = match kind {
        EnemyKind::Boss => 14,
        EnemyKind::Vortex => 5,
        EnemyKind::Tank | EnemyKind::Sniper | EnemyKind::Bomber => 3,
        EnemyKind::Swarmer | EnemyKind::Dasher => {
            if rng.range_u32(0, 100) < 32 {
                1
            } else {
                0
            }
        }
    };
    let mut xp: f32 = match kind {
        EnemyKind::Boss => 55.0_f32,
        EnemyKind::Vortex => 24.0_f32,
        EnemyKind::Tank => 16.0_f32,
        EnemyKind::Sniper | EnemyKind::Bomber => 14.0_f32,
        EnemyKind::Swarmer | EnemyKind::Dasher => 6.0_f32,
    };
    if elite {
        shard = shard.saturating_mul(3).max(3);
        xp = (xp * 2.4_f32).round();
    }
    KillReward { xp, shard }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn critical_hit_doubles_damage_when_chance_is_one() {
        let mut rng = GameRng::seeded(1);
        let hit = roll_hit_damage(25.0, 1.0, true, &mut rng);
        assert!(hit.is_crit);
        assert_eq!(hit.damage, 50.0);

        let no_crit = roll_hit_damage(25.0, 1.0, false, &mut rng);
        assert!(!no_crit.is_crit);
        assert_eq!(no_crit.damage, 25.0);
    }

    #[test]
    fn non_finite_or_negative_damage_is_ignored() {
        let mut rng = GameRng::seeded(2);
        assert_eq!(roll_hit_damage(f32::NAN, 0.5, true, &mut rng).damage, 0.0);
        assert_eq!(roll_hit_damage(-10.0, 0.5, true, &mut rng).damage, 0.0);

        let mut p = PlayerState::default();
        let out = apply_player_damage(&mut p, f32::NAN);
        assert!(!out.lethal);
        assert_eq!(p.hp, 100.0);
    }

    #[test]
    fn plasma_shield_absorbs_before_hull_and_resets_delay() {
        let mut p = PlayerState {
            hp: 100.0,
            max_hp: 100.0,
            shield: 30.0,
            max_shield: 60.0,
            shield_delay: 0.0,
            ..PlayerState::default()
        };

        let r1 = apply_player_damage(&mut p, 20.0);
        assert_eq!(r1.shield_absorbed, 20.0);
        assert_eq!(r1.hp_lost, 0.0);
        assert!(!r1.lethal);
        assert_eq!(p.shield, 10.0);
        assert_eq!(p.shield_delay, 3.8);

        let r2 = apply_player_damage(&mut p, 35.0);
        assert_eq!(r2.shield_absorbed, 10.0);
        assert_eq!(r2.hp_lost, 25.0);
        assert_eq!(p.shield, 0.0);
        assert_eq!(p.hp, 75.0);

        let r3 = apply_player_damage(&mut p, 999.0);
        assert!(r3.lethal);
        assert_eq!(p.hp, 0.0);
    }

    #[test]
    fn elite_rewards_scale_shards_and_xp_properly() {
        let mut rng = GameRng::seeded(42);
        let r = roll_kill_reward(EnemyKind::Tank, true, &mut rng);
        assert_eq!(r.shard, 9);
        assert_eq!(r.xp, 38.0);
    }
}
