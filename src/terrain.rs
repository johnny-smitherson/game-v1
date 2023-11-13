use bevy::prelude::Vec3;
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use smart_default::SmartDefault;

pub const PLANET_RADIUS: f32 = 100000.0;
pub const PLANET_MAX_PLAY_RADIUS: f32 = PLANET_RADIUS / 5.0;

pub const BASE_SPLIT_LEVEL: u8 = 4;

#[allow(non_snake_case)]
#[derive(Debug, Copy, Clone, Reflect, InspectorOptions, SmartDefault)]
#[reflect(InspectorOptions)]
pub struct TerrainSettings {
    #[default(20)]
    #[inspector(min=BASE_SPLIT_LEVEL, max=30)]
    pub MAX_SPLIT_LEVEL: u8,

    #[default(BASE_SPLIT_LEVEL+2)]
    #[inspector(min=BASE_SPLIT_LEVEL, max=10)]
    pub MIN_SPLIT_LEVEL: u8,

    #[default(1.8)]
    #[inspector(min = 1.0, max = 10.0)]
    pub TESSELATION_VALUE: f32,

    #[default(0.3)]
    #[inspector(min = 0.3, max = 3.0)]
    pub MIN_CAMERA_HEIGHT: f32,

    #[default(450.0)]
    #[inspector(min = 100.0, max = 500.0)]
    pub MAX_CAMERA_HEIGHT: f32,

    #[default(0.2)]
    #[inspector(min = 0.0, max = 0.45)]
    pub SPLIT_LAZY_COEF: f32,

    #[default(9.1)]
    #[inspector(min = 0.1, max = 20.0)]
    pub MIN_TRIANGLE_EDGE_SIZE: f32,
}

pub fn height(_pos: &Vec3) -> f32 {
    let (x, y) = (_pos.x, _pos.z);
    d_height(x, y)
}

pub fn apply_height(pos: &Vec3) -> Vec3 {
    Vec3::new(pos.x, height(pos), pos.z)
}

pub const NOISE_SEED: i32 = 11;
pub const MOUNTAIN_HEIGHT: f32 = 500.0;
pub const NOISE_BASE_FREQ: f32 = 100.0;

/// returns single noise value for unscaled position. noise is capped [-1, 1]
fn noise_single(x: f32, y: f32, seed: i32) -> f32 {
    use simdnoise::NoiseBuilder;
    // size = 1 does not work gives NaN
    NoiseBuilder::fbm_2d_offset(x, 2, y, 2)
        .with_seed(seed)
        .generate_scaled(-1.0, 1.0)[0]
}

fn d_height(x: f32, y: f32) -> f32 {
    let octaves = 2;
    let count_per_octave = 2;
    (1..=octaves)
        .map(|i| {
            let exp = 1.3_f32.powi(i);
            let freq = NOISE_BASE_FREQ * exp;
            let height = MOUNTAIN_HEIGHT / exp;
            (0..count_per_octave)
                .map(|j| noise_single(x / freq, y / freq, (j + 1) * NOISE_SEED + i) * height)
                .sum::<f32>()
        })
        .sum::<f32>()
        / octaves as f32
}
