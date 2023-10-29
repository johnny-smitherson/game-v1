use bevy::prelude::Vec3;
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use noise::{Billow, NoiseFn, Perlin};
use smart_default::SmartDefault;

pub const NOISE_SEED: f32 = 0.0;
pub const MOUNTAIN_HEIGHT: f32 = 1000.0;
pub const PLANET_RADIUS: f32 = 100000.0;

pub const NOISE_BASE_FREQ: f32 = 10000.0;
pub const BASE_SPLIT_LEVEL: u8 = 3;

#[allow(non_snake_case)]
#[derive(Debug, Copy, Clone, Reflect, InspectorOptions, SmartDefault)]
#[reflect(InspectorOptions)]
pub struct TerrainSettings {
    #[default(20)]
    #[inspector(min=BASE_SPLIT_LEVEL, max=30)]
    pub MAX_SPLIT_LEVEL: u8,

    #[default(BASE_SPLIT_LEVEL)]
    #[inspector(min=BASE_SPLIT_LEVEL, max=10)]
    pub MIN_SPLIT_LEVEL: u8,

    #[default(3.0)]
    #[inspector(min = 1.0, max = 10.0)]
    pub TESSELATION_VALUE: f32,

    #[default(0.3)]
    #[inspector(min = 0.3, max = 3.0)]
    pub MIN_CAMERA_HEIGHT: f32,

    #[default(250.0)]
    #[inspector(min = 100.0, max = 500.0)]
    pub MAX_CAMERA_HEIGHT: f32,

    #[default(0.2)]
    #[inspector(min = 0.0, max = 0.45)]
    pub SPLIT_LAZY_COEF: f32,

    #[default(5.2)]
    #[inspector(min = 0.1, max = 20.0)]
    pub MIN_TRIANGLE_EDGE_SIZE: f32,
}

pub fn height(_pos: &Vec3) -> f32 {
    // let perlin: Perlin = Perlin::new(3);
    let noise1 = Billow::<Perlin>::new(0);
    let noise2 = Billow::<Perlin>::new(1);
    let noise3 = Billow::<Perlin>::new(3);
    let noise4 = Billow::<Perlin>::new(2);
    let ref_pos = Vec3::new(_pos.x, NOISE_SEED, _pos.z) / NOISE_BASE_FREQ;
    let ret_val = (noise1.get([
        (ref_pos.x / 1.0) as f64,
        (ref_pos.y / 1.0) as f64,
        (ref_pos.z / 1.0) as f64,
    ]) + noise2.get([
        (ref_pos.x / 3.0) as f64,
        (ref_pos.y / 3.0) as f64,
        (ref_pos.z / 3.0) as f64,
    ]) + noise3.get([
        (ref_pos.x / 9.0) as f64,
        (ref_pos.y / 9.0) as f64,
        (ref_pos.z / 9.0) as f64,
    ]) + noise4.get([
        (ref_pos.x / 27.0) as f64,
        (ref_pos.y / 27.0) as f64,
        (ref_pos.z / 27.0) as f64,
    ])) / 4.0;
    let ret_val = ret_val as f32;

    MOUNTAIN_HEIGHT * ret_val
}

pub fn apply_height(pos: &Vec3) -> Vec3 {
    Vec3::new(pos.x, height(pos), pos.z)
}
