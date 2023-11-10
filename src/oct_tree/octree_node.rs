use super::bounding_box::BoundingBox;
use super::data_point::DataPoint;
use bevy::math::Vec3;

#[derive(Clone)]
pub struct OctreeNode {
    pub boundary: BoundingBox,
    pub capacity: usize,
    pub data_points: Vec<DataPoint>,
    pub divided: bool,
    pub children: [Option<Box<OctreeNode>>; 8],
}

impl OctreeNode {
    pub fn new(boundary: BoundingBox, capacity: usize) -> Self {
        Self {
            boundary,
            capacity,
            data_points: Vec::new(),
            divided: false,
            children: [None, None, None, None, None, None, None, None],
        }
    }
    pub fn insert(&mut self, data_point: DataPoint) -> bool {
        if !self.boundary.contains_point(&data_point.position) {
            return false;
        }

        if self.data_points.len() < self.capacity {
            self.data_points.push(data_point);
            return true;
        }

        if !self.divided {
            self.subdivide();
        }

        for ref mut child_node in self.children.iter_mut().flatten() {
            if child_node.insert(data_point) {
                return true;
            }
        }

        false
    }
    pub fn remove_data_point(&mut self, id: u32) -> Option<DataPoint> {
        if let Some(index) = self.data_points.iter().position(|dp| dp.id == id) {
            let removed_data_point = self.data_points.remove(index);
            self.merge();
            return Some(removed_data_point);
        }

        if self.divided {
            for ref mut child_node in self.children.iter_mut().flatten() {
                if let Some(removed_data_point) = child_node.remove_data_point(id) {
                    self.merge();
                    return Some(removed_data_point);
                }
            }
        }

        None
    }

    pub fn subdivide(&mut self) {
        let cx = self.boundary.center.x;
        let cy = self.boundary.center.y;
        let cz = self.boundary.center.z;
        let hx = self.boundary.half_extents.x;
        let hy = self.boundary.half_extents.y;
        let hz = self.boundary.half_extents.z;

        for i in 0..8 {
            let center = Vec3::new(
                cx + (i & 1) as f32 * hx,
                cy + ((i >> 1) & 1) as f32 * hy,
                cz + ((i >> 2) & 1) as f32 * hz,
            );

            let boundary = BoundingBox {
                center,
                half_extents: self.boundary.half_extents * 0.5,
            };

            self.children[i] = Some(Box::new(OctreeNode::new(boundary, self.capacity)));
        }

        self.divided = true;
    }
    pub fn query(&self, region: &BoundingBox, results: &mut Vec<DataPoint>) {
        if !self.boundary.intersects(region) {
            return;
        }

        for data_point in &self.data_points {
            if region.contains_point(&data_point.position) {
                results.push(*data_point);
            }
        }

        if self.divided {
            for child_node in self.children.iter().flatten() {
                child_node.query(region, results);
            }
        }
    }

    pub fn query_within_radius(
        &self,
        center: Vec3,
        squared_radius: f32,
        region: &BoundingBox,
        results: &mut Vec<DataPoint>,
    ) {
        if !self.boundary.intersects(region) {
            return;
        }

        for data_point in &self.data_points {
            let squared_distance = (data_point.position - center).length_squared();
            if squared_distance <= squared_radius {
                results.push(*data_point);
            }
        }

        if self.divided {
            for child_node in self.children.iter().flatten() {
                child_node.query_within_radius(center, squared_radius, region, results);
            }
        }
    }
    pub fn merge(&mut self) {
        if self.divided {
            let mut total_points = self.data_points.len();
            for child_node in self.children.iter().flatten() {
                total_points += child_node.data_points.len();
            }

            if total_points <= self.capacity {
                for child in self.children.iter_mut() {
                    if let Some(ref mut child_node) = child {
                        self.data_points.append(&mut child_node.data_points);
                        *child = None;
                    }
                }
                self.divided = false;
            }
        }
    }
    pub fn grow(&mut self, data_point: &DataPoint) {
        let direction = (data_point.position - self.boundary.center).normalize();
        let new_center = self.boundary.center + direction * self.boundary.half_extents;
        let new_boundary = BoundingBox {
            center: new_center,
            half_extents: self.boundary.half_extents * 2.0,
        };

        let mut new_root = OctreeNode::new(new_boundary, self.capacity);
        new_root.insert(*data_point);
        new_root.divided = true;

        let child_index = new_root.get_child_index(&self.boundary.center);
        new_root.children[child_index] = Some(Box::new(self.clone()));

        *self = new_root;
    }

    fn get_child_index(&self, point: &Vec3) -> usize {
        let mut index = 0;

        if point.x >= self.boundary.center.x {
            index |= 1;
        }
        if point.y >= self.boundary.center.y {
            index |= 2;
        }
        if point.z >= self.boundary.center.z {
            index |= 4;
        }

        index
    }
}
