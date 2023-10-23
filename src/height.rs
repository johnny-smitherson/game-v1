use bevy::prelude::Vec3;
use noise::{Billow, NoiseFn, Perlin};

pub const MOUNTAIN_HEIGHT: f32 = 500.0;
pub const PLANET_RADIUS: f32 = 10000.0;
pub const NOISE_SEED: f32 = 100.0;

#[allow(non_snake_case)]
#[derive(Debug, Copy, Clone)]
pub struct TerrainSettings {
    pub MAX_SPLIT_LEVEL: u8,
    pub MIN_SPLIT_LEVEL: u8,
    pub TESSELATION_VALUE: f32,
    pub MIN_CAMERA_HEIGHT: f32,
    pub MAX_CAMERA_HEIGHT: f32,
    pub SPLIT_LAZY_COEF: f32,
    pub MIN_TRIANGLE_EDGE_SIZE: f32,
}
impl Default for TerrainSettings {
    fn default() -> Self {
        Self {
            MAX_SPLIT_LEVEL: 13,
            MIN_SPLIT_LEVEL: 2,
            TESSELATION_VALUE: 5.0,
            MIN_CAMERA_HEIGHT: 0.3,
            MAX_CAMERA_HEIGHT: 250.,
            // defines how lazy the split/merge operation is
            SPLIT_LAZY_COEF: 0.3,
            MIN_TRIANGLE_EDGE_SIZE: 1.0,
        }
    }
}

pub fn height(_pos: &Vec3) -> f32 {
    // let perlin: Perlin = Perlin::new(3);
    let noise1 = Billow::<Perlin>::new(0);
    let noise2 = Billow::<Perlin>::new(1);
    let noise3 = Billow::<Perlin>::new(3);
    let noise4 = Billow::<Perlin>::new(2);
    let ref_pos = Vec3::new(_pos.x, NOISE_SEED, _pos.z) / PLANET_RADIUS;
    let ret_val = (noise1.get([
        (ref_pos.x / 1.0) as f64,
        (ref_pos.y / 1.0) as f64,
        (ref_pos.z / 1.0) as f64,
    ]) + noise2.get([
        (ref_pos.x / 2.0) as f64,
        (ref_pos.y / 2.0) as f64,
        (ref_pos.z / 2.0) as f64,
    ]) + noise3.get([
        (ref_pos.x / 4.0) as f64,
        (ref_pos.y / 4.0) as f64,
        (ref_pos.z / 4.0) as f64,
    ]) + noise4.get([
        (ref_pos.x / 8.0) as f64,
        (ref_pos.y / 8.0) as f64,
        (ref_pos.z / 8.0) as f64,
    ])) / 4.0;
    let ret_val = ret_val as f32;

    MOUNTAIN_HEIGHT * ret_val
}

pub fn apply_height(pos: &Vec3) -> Vec3 {
    Vec3::new(pos.x, height(pos), pos.z)
}
