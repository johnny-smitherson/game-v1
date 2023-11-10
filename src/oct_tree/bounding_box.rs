use bevy::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
/// an axist aligned bounding box in 3d space
pub struct BoundingBox {
    pub center: Vec3,
    pub half_extents: Vec3,
}

impl BoundingBox {
    pub fn new(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            center,
            half_extents,
        }
    }
    pub fn new_from_point_and_radius(point: Vec3, radius: f32) -> Self {
        Self {
            center: point,
            half_extents: Vec3::new(radius, radius, radius),
        }
    }
    pub fn get_closest_point(&self, point: &Vec3) -> Vec3 {
        let mut closest_point = Vec3::ZERO;
        for i in 0..3 {
            let min = self.center[i] - self.half_extents[i];
            let max = self.center[i] + self.half_extents[i];
            if point[i] < min {
                closest_point[i] = min;
            } else if point[i] > max {
                closest_point[i] = max;
            } else {
                closest_point[i] = point[i];
            }
        }
        closest_point
    }
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        let closest_point = self.get_closest_point(&point);
        (closest_point - point).length()
    }
    pub fn zero() -> Self {
        Self {
            center: Vec3::ZERO,
            half_extents: Vec3::ZERO,
        }
    }
    pub fn equals(&self, other: &BoundingBox) -> bool {
        self.center == other.center && self.half_extents == other.half_extents
    }
    pub fn merge_two_create_new(aabb_a: &Self, aabb_b: &Self) -> Self {
        let mut aabb = BoundingBox::zero();
        aabb.merge_with_aabb(aabb_a);
        aabb.merge_with_aabb(aabb_b);
        aabb
    }
    pub fn merged(&self, other: &BoundingBox) -> Self {
        let mut aabb = BoundingBox::zero();
        aabb.merge_with_aabb(self);
        aabb.merge_with_aabb(other);
        aabb
    }
    pub fn merge_with_aabb(&mut self, other: &BoundingBox) {
        self.center.x = self.center.x.min(other.center.x);
        self.center.y = self.center.y.min(other.center.y);
        self.center.z = self.center.z.min(other.center.z);
        self.half_extents.x = self.half_extents.x.max(other.half_extents.x);
        self.half_extents.y = self.half_extents.y.max(other.half_extents.y);
        self.half_extents.z = self.half_extents.z.max(other.half_extents.z);
    }
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.center.x <= other.half_extents.x
            && self.half_extents.x >= other.center.x
            && self.center.y <= other.half_extents.y
            && self.half_extents.y >= other.center.y
            && self.center.z <= other.half_extents.z
            && self.half_extents.z >= other.center.z
    }
    pub fn join_and_make_new(&self, other: &BoundingBox) -> Self {
        let mut aabb = BoundingBox::zero();
        aabb.merge_with_aabb(self);
        aabb.merge_with_aabb(other);
        aabb
    }
    pub fn contains_point(&self, point: &Vec3) -> bool {
        let min = self.center - self.half_extents;
        let max = self.center + self.half_extents;
        point.x >= min.x
            && point.x <= max.x
            && point.y >= min.y
            && point.y <= max.y
            && point.z >= min.z
            && point.z <= max.z
    }

    pub fn get_random_point_within(&self) -> Vec3 {
        let min = self.center - self.half_extents;
        let max = self.center + self.half_extents;
        Vec3::new(
            rand::random::<f32>() * (max.x - min.x) + min.x,
            rand::random::<f32>() * (max.y - min.y) + min.y,
            rand::random::<f32>() * (max.z - min.z) + min.z,
        )
    }

    pub fn max(&self) -> Vec3 {
        self.center + self.half_extents
    }

    pub fn min(&self) -> Vec3 {
        self.center - self.half_extents
    }

    pub fn center(&self) -> Vec3 {
        self.center
    }

    pub fn set_x_min(&mut self, x: f32) {
        self.center.x = x + self.half_extents.x;
    }

    pub fn set_x_max(&mut self, x: f32) {
        self.center.x = x - self.half_extents.x;
    }

    pub fn set_y_min(&mut self, y: f32) {
        self.center.y = y + self.half_extents.y;
    }

    pub fn set_y_max(&mut self, y: f32) {
        self.center.y = y - self.half_extents.y;
    }

    pub fn set_z_min(&mut self, z: f32) {
        self.center.z = z + self.half_extents.z;
    }

    pub fn set_z_max(&mut self, z: f32) {
        self.center.z = z - self.half_extents.z;
    }
}
