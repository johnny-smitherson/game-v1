use bevy::math::Vec3;

#[derive(Debug, Clone, Copy)]

/// a point in 3d space with a unique id
pub struct DataPoint {
  pub id: u32,
  pub position: Vec3,
}

impl DataPoint {
  pub fn new(id: u32, position: Vec3) -> Self {
    Self { id, position }
  }
}