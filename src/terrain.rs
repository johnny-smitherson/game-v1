use bevy::prelude::Vec3;
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use smart_default::SmartDefault;

pub const PLANET_RADIUS: f32 = 100000.0;

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
    let (x, y) = (_pos.x, _pos.z);
    ::game_terrain::height(x, y)
}

pub fn apply_height(pos: &Vec3) -> Vec3 {
    Vec3::new(pos.x, height(pos), pos.z)
}
