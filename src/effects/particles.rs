//! Particle, expanding ring, and lightning arc primitives.

use macroquad::color::Color;
use macroquad::math::Vec2;

#[derive(Clone, Copy)]
pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub color: Color,
}

#[derive(Clone, Copy)]
pub struct Ring {
    pub pos: Vec2,
    pub radius: f32,
    pub max_radius: f32,
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
}

#[derive(Clone, Copy)]
pub struct ArcBolt {
    pub a: Vec2,
    pub b: Vec2,
    pub life: f32,
}
