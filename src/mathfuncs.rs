use glam::{Vec2};
use rand::prelude::*;

pub fn vec_from_angle(angle: f32) -> Vec2 {
    let vx = angle.sin();
    let vy = angle.cos();
    Vec2::new(vx, vy)
}

pub fn generate_random_scoords(min: f32, max: f32) -> f32 {
    let res = rand::thread_rng().gen_range(min..max).floor();
    res
}