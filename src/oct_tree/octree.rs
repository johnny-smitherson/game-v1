use super::bounding_box::BoundingBox;
use super::data_point::DataPoint;
use super::octree_node::OctreeNode;
use bevy::math::Vec3;

pub struct Octree {
    root: OctreeNode,
}

impl Octree {
    pub fn new(boundary: BoundingBox, capacity: usize) -> Self {
        Self {
            root: OctreeNode::new(boundary, capacity),
        }
    }
    pub fn insert(&mut self, data_point: DataPoint) -> bool {
        self.root.insert(data_point)
    }
    pub fn query(&self, region: &BoundingBox, results: &mut Vec<DataPoint>) {
        self.root.query(region, results)
    }
    pub fn update_position(&mut self, id: u32, new_position: Vec3) -> bool {
        if let Some(mut data_point) = self.root.remove_data_point(id) {
            if !self.root.boundary.contains_point(&new_position) {
                self.root.grow(&data_point)
            }

            data_point.position = new_position;
            self.insert(data_point)
        } else {
            false
        }
    }
    pub fn query_within_radius(&self, center: &Vec3, radius: f32, results: &mut Vec<DataPoint>) {
        let half_extents = Vec3::splat(radius);
        let region = BoundingBox {
            center: *center,
            half_extents,
        };
        self.root
            .query_within_radius(*center, radius * radius, &region, results);
    }

    pub fn get_root_boundary(&self) -> BoundingBox {
        self.root.boundary
    }

    pub fn get_root_center(&self) -> Vec3 {
        self.root.boundary.center
    }
}

#[test]
fn test_basic_operation() {
    let mut octree = Octree::new(BoundingBox::new(Vec3::ZERO, Vec3::splat(100.0)), 4);
    let data_points = vec![
        DataPoint::new(0, Vec3::new(0.0, 0.0, 0.0)),
        DataPoint::new(1, Vec3::new(1.0, 1.0, 1.0)),
        DataPoint::new(2, Vec3::new(2.0, 2.0, 2.0)),
        DataPoint::new(3, Vec3::new(3.0, 3.0, 3.0)),
        DataPoint::new(4, Vec3::new(4.0, 4.0, 4.0)),
        DataPoint::new(5, Vec3::new(5.0, 5.0, 5.0)),
        DataPoint::new(6, Vec3::new(6.0, 6.0, 6.0)),
        DataPoint::new(7, Vec3::new(7.0, 7.0, 7.0)),
        DataPoint::new(8, Vec3::new(8.0, 8.0, 8.0)),
        DataPoint::new(9, Vec3::new(9.0, 9.0, 9.0)),
        DataPoint::new(10, Vec3::new(10.0, 10.0, 10.0)),
        DataPoint::new(11, Vec3::new(11.0, 11.0, 11.0)),
        DataPoint::new(12, Vec3::new(12.0, 12.0, 12.0)),
        DataPoint::new(13, Vec3::new(13.0, 13.0, 13.0)),
        DataPoint::new(14, Vec3::new(14.0, 14.0, 14.0)),
        DataPoint::new(15, Vec3::new(15.0, 15.0, 15.0)),
        DataPoint::new(16, Vec3::new(16.0, 16.0, 16.0)),
        DataPoint::new(17, Vec3::new(17.0, 17.0, 17.0)),
        DataPoint::new(18, Vec3::new(18.0, 18.0, 18.0)),
        DataPoint::new(19, Vec3::new(19.0, 19.0, 19.0)),
        DataPoint::new(20, Vec3::new(20.0, 20.0, 20.0)),
    ];

    for data_point in data_points {
        octree.insert(data_point);
    }

    let mut results = vec![];

    octree.query(
        &BoundingBox::new(Vec3::new(0.0, 0.0, 0.0), Vec3::splat(10.0)),
        &mut results,
    );

    assert_eq!(results.len(), 11);

    results.clear();

    octree.query(
        &BoundingBox::new(Vec3::new(0.0, 0.0, 0.0), Vec3::splat(5.0)),
        &mut results,
    );

    assert_eq!(results.len(), 6);

    results.clear();

    octree.query(
        &BoundingBox::new(Vec3::new(0.0, 0.0, 0.0), Vec3::splat(1.0)),
        &mut results,
    );

    assert_eq!(results.len(), 2);

    results.clear();

    octree.query(
        &BoundingBox::new(Vec3::new(0.0, 0.0, 0.0), Vec3::splat(0.1)),
        &mut results,
    );

    assert_eq!(results.len(), 1);

    results.clear();

    octree.query(
        &BoundingBox::new(Vec3::new(0.0, 0.0, 0.0), Vec3::splat(100.0)),
        &mut results,
    );

    assert_eq!(results.len(), 19);
}

#[test]
fn test_query_within_radius() {
    let mut octree = Octree::new(BoundingBox::new(Vec3::ZERO, Vec3::splat(100.0)), 4);
    let data_points = vec![
        DataPoint::new(0, Vec3::new(0.0, 0.0, 0.0)),
        DataPoint::new(1, Vec3::new(1.0, 1.0, 1.0)),
        DataPoint::new(2, Vec3::new(2.0, 2.0, 2.0)),
        DataPoint::new(3, Vec3::new(3.0, 3.0, 3.0)),
        DataPoint::new(4, Vec3::new(4.0, 4.0, 4.0)),
        DataPoint::new(5, Vec3::new(5.0, 5.0, 5.0)),
        DataPoint::new(6, Vec3::new(6.0, 6.0, 6.0)),
        DataPoint::new(7, Vec3::new(7.0, 7.0, 7.0)),
        DataPoint::new(8, Vec3::new(8.0, 8.0, 8.0)),
        DataPoint::new(9, Vec3::new(9.0, 9.0, 9.0)),
        DataPoint::new(10, Vec3::new(10.0, 10.0, 10.0)),
        DataPoint::new(11, Vec3::new(11.0, 11.0, 11.0)),
        DataPoint::new(12, Vec3::new(12.0, 12.0, 12.0)),
        DataPoint::new(13, Vec3::new(13.0, 13.0, 13.0)),
        DataPoint::new(14, Vec3::new(14.0, 14.0, 14.0)),
        DataPoint::new(15, Vec3::new(15.0, 15.0, 15.0)),
        DataPoint::new(16, Vec3::new(16.0, 16.0, 16.0)),
        DataPoint::new(17, Vec3::new(17.0, 17.0, 17.0)),
        DataPoint::new(18, Vec3::new(18.0, 18.0, 18.0)),
        DataPoint::new(19, Vec3::new(19.0, 19.0, 19.0)),
        DataPoint::new(20, Vec3::new(20.0, 20.0, 20.0)),
    ];

    for data_point in data_points {
        octree.insert(data_point);
    }

    let mut results = vec![];

    octree.query_within_radius(&Vec3::new(0.0, 0.0, 0.0), 10.0, &mut results);

    assert_eq!(results.len(), 6);

    results.clear();

    octree.query_within_radius(&Vec3::new(0.0, 0.0, 0.0), 5.0, &mut results);

    assert_eq!(results.len(), 3);

    results.clear();

    octree.query_within_radius(&Vec3::new(0.0, 0.0, 0.0), 1.0, &mut results);

    assert_eq!(results.len(), 1);

    results.clear();

    octree.query_within_radius(&Vec3::new(0.0, 0.0, 0.0), 0.1, &mut results);

    assert_eq!(results.len(), 1);

    results.clear();

    octree.query_within_radius(&Vec3::new(0.0, 0.0, 0.0), 100.0, &mut results);

    assert_eq!(results.len(), 19);
}

#[test]
fn test_update_position() {
    let mut octree = Octree::new(BoundingBox::new(Vec3::ZERO, Vec3::splat(100.0)), 4);
    let data_points = vec![
        DataPoint::new(0, Vec3::new(0.0, 0.0, 0.0)),
        DataPoint::new(1, Vec3::new(1.0, 1.0, 1.0)),
        DataPoint::new(2, Vec3::new(2.0, 2.0, 2.0)),
        DataPoint::new(3, Vec3::new(3.0, 3.0, 3.0)),
        DataPoint::new(4, Vec3::new(4.0, 4.0, 4.0)),
        DataPoint::new(5, Vec3::new(5.0, 5.0, 5.0)),
        DataPoint::new(6, Vec3::new(6.0, 6.0, 6.0)),
        DataPoint::new(7, Vec3::new(7.0, 7.0, 7.0)),
        DataPoint::new(8, Vec3::new(8.0, 8.0, 8.0)),
        DataPoint::new(9, Vec3::new(9.0, 9.0, 9.0)),
        DataPoint::new(10, Vec3::new(10.0, 10.0, 10.0)),
        DataPoint::new(11, Vec3::new(11.0, 11.0, 11.0)),
        DataPoint::new(12, Vec3::new(12.0, 12.0, 12.0)),
        DataPoint::new(13, Vec3::new(13.0, 13.0, 13.0)),
        DataPoint::new(14, Vec3::new(14.0, 14.0, 14.0)),
        DataPoint::new(15, Vec3::new(15.0, 15.0, 15.0)),
        DataPoint::new(16, Vec3::new(16.0, 16.0, 16.0)),
        DataPoint::new(17, Vec3::new(17.0, 17.0, 17.0)),
        DataPoint::new(18, Vec3::new(18.0, 18.0, 18.0)),
        DataPoint::new(19, Vec3::new(19.0, 19.0, 19.0)),
        DataPoint::new(20, Vec3::new(20.0, 20.0, 20.0)),
    ];

    for data_point in data_points {
        octree.insert(data_point);
    }

    // Move the first point to the end of the octree

    octree.update_position(0, Vec3::new(20.0, 20.0, 20.0));

    let mut results = vec![];

    octree.query_within_radius(&Vec3::new(0.0, 0.0, 0.0), 10.0, &mut results);

    assert_eq!(results.len(), 5);

    results.clear();

    octree.query_within_radius(&Vec3::new(0.0, 0.0, 0.0), 5.0, &mut results);

    assert_eq!(results.len(), 2);

    results.clear();

    octree.query_within_radius(&Vec3::new(0.0, 0.0, 0.0), 1.0, &mut results);

    assert_eq!(results.len(), 0);

    results.clear();

    octree.query_within_radius(&Vec3::new(0.0, 0.0, 0.0), 0.1, &mut results);

    assert_eq!(results.len(), 0);

    results.clear();

    octree.query_within_radius(&Vec3::new(0.0, 0.0, 0.0), 100.0, &mut results);

    assert_eq!(results.len(), 19);

    // Move the first point back to the beginning of the octree

    octree.update_position(0, Vec3::new(0.0, 0.0, 0.0));

    results.clear();

    octree.query_within_radius(&Vec3::new(0.0, 0.0, 0.0), 10.0, &mut results);

    assert_eq!(results.len(), 6);

    results.clear();

    octree.query_within_radius(&Vec3::new(0.0, 0.0, 0.0), 5.0, &mut results);

    assert_eq!(results.len(), 3);

    results.clear();

    octree.query_within_radius(&Vec3::new(0.0, 0.0, 0.0), 1.0, &mut results);

    assert_eq!(results.len(), 1);

    results.clear();

    octree.query_within_radius(&Vec3::new(0.0, 0.0, 0.0), 0.1, &mut results);

    assert_eq!(results.len(), 1);

    results.clear();

    octree.query_within_radius(&Vec3::new(0.0, 0.0, 0.0), 100.0, &mut results);

    assert_eq!(results.len(), 19);
}
