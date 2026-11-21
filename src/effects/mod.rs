//! Visual effects pool: particles, rings, arcs, floating texts, screen shake, hit-stop.

pub mod floating_text;
pub mod particles;

pub use floating_text::{DmgText, TextBuf};
pub use particles::{ArcBolt, Particle, Ring};

use crate::game::config::{MAX_ARCS, MAX_PARTICLES, MAX_RINGS, MAX_TEXTS};
use macroquad::color::Color;
use macroquad::math::Vec2;

pub struct EffectsPool {
    pub particles: Vec<Particle>,
    pub rings: Vec<Ring>,
    pub texts: Vec<DmgText>,
    pub arcs: Vec<ArcBolt>,
    pub max_particles: usize,
    pub shake: f32,
    pub hit_stop: f32,
    pub hurt_flash: f32,
}

impl EffectsPool {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(MAX_PARTICLES),
            rings: Vec::with_capacity(MAX_RINGS),
            texts: Vec::with_capacity(MAX_TEXTS),
            arcs: Vec::with_capacity(MAX_ARCS),
            max_particles: MAX_PARTICLES,
            shake: 0.0,
            hit_stop: 0.0,
            hurt_flash: 0.0,
        }
    }

    pub fn set_particle_cap(&mut self, cap: usize) {
        self.max_particles = cap.clamp(32, MAX_PARTICLES);
        if self.particles.len() > self.max_particles {
            self.particles.truncate(self.max_particles);
        }
    }

    pub fn clear(&mut self) {
        self.particles.clear();
        self.rings.clear();
        self.texts.clear();
        self.arcs.clear();
        self.shake = 0.0;
        self.hit_stop = 0.0;
        self.hurt_flash = 0.0;
    }

    pub fn spawn_particle(&mut self, pos: Vec2, vel: Vec2, life: f32, size: f32, color: Color) {
        if self.particles.len() >= self.max_particles {
            return;
        }
        self.particles.push(Particle {
            pos,
            vel,
            life,
            max_life: life.max(0.001),
            size,
            color,
        });
    }

    pub fn spawn_ring(&mut self, pos: Vec2, max_radius: f32, color: Color, life: f32) {
        if self.rings.len() >= MAX_RINGS {
            return;
        }
        self.rings.push(Ring {
            pos,
            radius: 4.0,
            max_radius,
            life,
            max_life: life.max(0.001),
            color,
        });
    }

    pub fn spawn_text(&mut self, pos: Vec2, val: TextBuf<32>, color: Color) {
        if self.texts.len() >= MAX_TEXTS {
            return;
        }
        self.texts.push(DmgText {
            pos,
            val,
            life: 0.55,
            color,
        });
    }

    pub fn spawn_arc(&mut self, a: Vec2, b: Vec2) {
        if self.arcs.len() >= MAX_ARCS {
            return;
        }
        self.arcs.push(ArcBolt { a, b, life: 0.14 });
    }

    /// Applies camera shake scaled by `shake_scale` (`0.0`, `0.5`, or `1.0`).
    pub fn add_shake(&mut self, amount: f32, shake_scale: f32) {
        if shake_scale > 0.0 && amount.is_finite() && shake_scale.is_finite() {
            self.shake = self.shake.max(amount * shake_scale.clamp(0.0, 1.0));
        }
    }

    /// Schedules impact freeze-frame when `enabled` is true.
    pub fn add_hit_stop(&mut self, duration: f32, enabled: bool) {
        if enabled && duration.is_finite() && duration > 0.0 {
            self.hit_stop = self.hit_stop.max(duration);
        }
    }

    /// Consumes hit-stop time. Returns `true` if simulation should pause this frame.
    pub fn consume_hit_stop(&mut self, dt: f32) -> bool {
        if self.hit_stop > 0.0 {
            self.hit_stop = (self.hit_stop - dt).max(0.0);
            if self.shake > 0.0 {
                self.shake = (self.shake - dt * 22.0).max(0.0);
            }
            return true;
        }
        false
    }

    pub fn update(&mut self, dt: f32) {
        if self.hurt_flash > 0.0 {
            self.hurt_flash = (self.hurt_flash - dt * 2.8).max(0.0);
        }

        let mut pi = self.particles.len();
        while pi > 0 {
            pi -= 1;
            let vel = self.particles[pi].vel;
            self.particles[pi].pos += vel * dt;
            self.particles[pi].life -= dt;
            if self.particles[pi].life <= 0.0 {
                self.particles.swap_remove(pi);
            }
        }

        let mut ri = self.rings.len();
        while ri > 0 {
            ri -= 1;
            self.rings[ri].life -= dt;
            let ratio = (self.rings[ri].life / self.rings[ri].max_life).max(0.0);
            self.rings[ri].radius = self.rings[ri].max_radius * (1.0 - ratio);
            if self.rings[ri].life <= 0.0 {
                self.rings.swap_remove(ri);
            }
        }

        let mut ti = self.texts.len();
        while ti > 0 {
            ti -= 1;
            self.texts[ti].pos.y -= 52.0 * dt;
            self.texts[ti].life -= dt;
            if self.texts[ti].life <= 0.0 {
                self.texts.swap_remove(ti);
            }
        }

        let mut ai = self.arcs.len();
        while ai > 0 {
            ai -= 1;
            self.arcs[ai].life -= dt;
            if self.arcs[ai].life <= 0.0 {
                self.arcs.swap_remove(ai);
            }
        }

        if self.shake > 0.0 {
            self.shake = (self.shake - dt * 30.0).max(0.0);
        }
    }
}

impl Default for EffectsPool {
    fn default() -> Self {
        Self::new()
    }
}
