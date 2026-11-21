//! Player bullets, enemy projectiles, and Phantom Wraith proximity mines.

use macroquad::math::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct Bullet {
    pub pos: Vec2,
    pub vel: Vec2,
    pub dmg: f32,
    pub life: f32,
    pub pierce: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct EnemyBullet {
    pub pos: Vec2,
    pub vel: Vec2,
    pub dmg: f32,
    pub life: f32,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Mine {
    pub pos: Vec2,
    pub life: f32,
    pub dmg: f32,
    pub radius: f32,
}
