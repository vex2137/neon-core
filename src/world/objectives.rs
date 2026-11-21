//! Overload Beacon interactive map objective.

use super::arena::clamp_to_arena;
use crate::rng::GameRng;
use macroquad::math::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct BeaconObjective {
    pub active: bool,
    pub pos: Vec2,
    pub radius: f32,
    pub progress: f32,
    pub duration: f32,
    pub ttl: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BeaconTickEvent {
    None,
    Spawned,
    Captured,
    Expired,
}

impl Default for BeaconObjective {
    fn default() -> Self {
        Self {
            active: false,
            pos: Vec2::ZERO,
            radius: 82.0,
            progress: 0.0,
            duration: 2.4,
            ttl: 0.0,
        }
    }
}

impl BeaconObjective {
    pub fn reset(&mut self) {
        self.active = false;
        self.progress = 0.0;
        self.ttl = 0.0;
    }

    pub fn spawn_near(&mut self, player_pos: Vec2, rng: &mut GameRng) {
        let a = rng.range_f32(0.0, std::f32::consts::TAU);
        let dist = rng.range_f32(540.0, 1020.0);
        let pos = clamp_to_arena(player_pos + Vec2::from_angle(a) * dist, 260.0);
        *self = Self {
            active: true,
            pos,
            radius: 82.0,
            progress: 0.0,
            duration: 2.4,
            ttl: 28.0,
        };
    }

    pub fn tick(
        &mut self,
        spawn_timer: &mut f32,
        dt: f32,
        player_pos: Vec2,
        rng: &mut GameRng,
    ) -> BeaconTickEvent {
        if !self.active {
            *spawn_timer -= dt;
            if *spawn_timer <= 0.0 {
                self.spawn_near(player_pos, rng);
                return BeaconTickEvent::Spawned;
            }
            return BeaconTickEvent::None;
        }

        self.ttl -= dt;
        let b_r2 = self.radius * self.radius;
        if player_pos.distance_squared(self.pos) <= b_r2 {
            self.progress += dt;
            if self.progress >= self.duration {
                return BeaconTickEvent::Captured;
            }
        } else {
            self.progress = (self.progress - dt * 0.45).max(0.0);
        }

        if self.ttl <= 0.0 {
            self.active = false;
            self.progress = 0.0;
            *spawn_timer = 18.0;
            return BeaconTickEvent::Expired;
        }
        BeaconTickEvent::None
    }
}
