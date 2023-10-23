// use std::collections::{vec_deque, VecDeque};

use bevy::prelude::*;
use rand::Rng;

use bevy::render::mesh::{Indices, PrimitiveTopology};
// use rayon::prelude::IntoParallelRefMutIterator;

use super::height::apply_height;
use crate::height::TerrainSettings;

#[derive(Debug, Clone, Copy)]
pub struct TriangleData {
    verts: [Vec3; 3],
    uvs: [Vec2; 3],
    norm: [Vec3; 3],
    center: Vec3,
    max_edge_len: f32,
    min_edge_len: f32,
}
impl TriangleData {
    fn new(verts: [Vec3; 3]) -> Self {
        let mut rng = rand::thread_rng();
        let uv_x = rng.gen_range(0.0..1.0) as f32;
        let uv_y = rng.gen_range(0.0..1.0) as f32;

        let v12 = verts[2] - verts[1];
        let v01 = verts[1] - verts[0];
        let norm = -v12.cross(v01).normalize();
        let normals = [norm, norm, norm];

        let l1 = (verts[0] - verts[1]).length();
        let l2 = (verts[2] - verts[1]).length();
        let l3 = (verts[0] - verts[2]).length();

        // let uvs = [
        //     Vec2 { x: 0.0, y: 0.0 },
        //     Vec2 { x: 1.0, y: 0.0 },
        //     Vec2 { x: 1.0, y: 1.0 },
        // ];
        let uvs = [
            Vec2 { x: uv_x, y: uv_y },
            Vec2 { x: uv_x, y: uv_y },
            Vec2 { x: uv_x, y: uv_y },
        ];

        Self {
            verts,
            uvs,
            norm: normals,
            center: (verts[0] + verts[1] + verts[2]) / 3.0,
            max_edge_len: max3(l1, l2, l3),
            min_edge_len: min3(l1, l2, l3),
        }
    }
}

/// Triangle made of 3 vec3 corners
#[derive(Debug, Clone, Component)]
pub struct Triangle {
    data: TriangleData,
    all_data: Vec<TriangleData>,
    pub level: u8,
    pub id: u8,
    pub children: Option<[Box<Triangle>; 4]>,
    // pub dirty: bool,
    // pub parent_triangle: Option<Box<Triangle>>,
}

impl Default for Triangle {
    fn default() -> Self {
        Triangle::new(
            [
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, -1.0),
            ],
            0,
            0,
        )
    }
}

fn max3(l1: f32, l2: f32, l3: f32) -> f32 {
    if l1 > l2 {
        if l1 > l3 {
            l1
        } else {
            l3
        }
    } else if l2 > l3 {
        l2
    } else {
        l3
    }
}

fn min3(l1: f32, l2: f32, l3: f32) -> f32 {
    if l1 < l2 {
        if l1 < l3 {
            l1
        } else {
            l3
        }
    } else if l2 < l3 {
        l2
    } else {
        l3
    }
}

impl Triangle {
    pub fn new(points: [Vec3; 3], level: u8, id: u8) -> Self {
        let data = TriangleData::new([
            apply_height(&points[0]),
            apply_height(&points[1]),
            apply_height(&points[2]),
        ]);
        Self {
            data,
            all_data: vec![data],
            level,
            id,
            children: None,
        }
    }
    pub fn reverse_points(&self) -> Self {
        Self::new(
            [self.data.verts[1], self.data.verts[0], self.data.verts[2]],
            self.level,
            self.id,
        )
    }

    pub fn generate_mesh(&self, _settings: &TerrainSettings) -> Mesh {
        let mut all_verts = Vec::<Vec3>::new();
        let mut all_norms = Vec::<Vec3>::new();
        let mut all_uvs = Vec::<Vec2>::new();
        let mut all_indices = Vec::<u32>::new();
        let mut idx: u32 = 0;
        for data in self.all_data.iter() {
            all_verts.extend_from_slice(&data.verts);
            all_norms.extend_from_slice(&data.norm);
            all_uvs.extend_from_slice(&data.uvs);
            all_indices.extend_from_slice(&[idx, idx + 1, idx + 2]);
            idx += 3;
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(all_indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, all_verts);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, all_norms);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, all_uvs);
        mesh
    }

    pub fn is_split(&self) -> bool {
        self.children.is_some()
    }

    fn split(&mut self) {
        assert!(!self.is_split(), "can't split with children");
        let [v1, v2, v3] = self.data.verts;
        let v12 = (v1 + v2) * 0.5;
        let v23 = (v2 + v3) * 0.5;
        let v13 = (v1 + v3) * 0.5;

        self.children = Some([
            Box::new(Triangle::new([v12, v23, v13], self.level + 1, 1)),
            Box::new(Triangle::new([v1, v12, v13], self.level + 1, 2)),
            Box::new(Triangle::new([v12, v2, v23], self.level + 1, 3)),
            Box::new(Triangle::new([v13, v23, v3], self.level + 1, 4)),
        ]);
    }
    fn merge(&mut self) {
        assert!(self.is_split(), "can't mrege without children");
        self.children = None;
    }

    /// returns true if we changed something notable and you wanna update the thing
    pub fn update_split(&mut self, pos: &Vec3, settings: &TerrainSettings) -> bool {
        use rayon::prelude::*;

        let mut dirty: bool = false;
        if !self.is_split() && self.should_split(pos, settings) {
            self.split();
            dirty = true;
        }
        if self.is_split() && self.should_merge(pos, settings) {
            self.merge();
            dirty = true;
        }
        // triger children
        if self.is_split() {
            let child_results: Vec<bool> = self
                .children
                .as_mut()
                .expect("wtf")
                .par_iter_mut()
                .map(|child| child.as_mut().update_split(pos, settings))
                .collect();

            for child_dirty in child_results {
                dirty = child_dirty || dirty;
            }
        }
        // pull data from children
        if dirty {
            self.all_data = Vec::<TriangleData>::new();
            if self.is_split() {
                for child in self.children.as_ref().expect("wtf").iter() {
                    self.all_data.extend(child.all_data.iter());
                }
            } else {
                self.all_data.push(self.data);
            }
        }
        dirty
    }

    pub fn tri_count(&self) -> usize {
        if !self.is_split() {
            1
        } else {
            let mut count: usize = 0;
            for child in self.children.as_ref().expect("msg").iter().as_ref() {
                count += child.tri_count();
            }
            count
        }
    }

    fn should_split(&self, pos: &Vec3, settings: &TerrainSettings) -> bool {
        if self.level < settings.MIN_SPLIT_LEVEL {
            return true;
        }
        if self.level >= settings.MAX_SPLIT_LEVEL
            || self.data.min_edge_len < settings.MIN_TRIANGLE_EDGE_SIZE
            || self.data.max_edge_len / self.data.min_edge_len > 2.0
        {
            false
        } else {
            self.get_distance_over_size(pos)
                < (1.0 - settings.SPLIT_LAZY_COEF) * settings.TESSELATION_VALUE
        }
    }
    fn should_merge(&self, pos: &Vec3, settings: &TerrainSettings) -> bool {
        if self.level > settings.MAX_SPLIT_LEVEL {
            true
        } else {
            self.get_distance_over_size(pos)
                > (1.0 + settings.SPLIT_LAZY_COEF) * settings.TESSELATION_VALUE
        }
    }

    fn get_distance_over_size(&self, pos: &Vec3) -> f32 {
        (*pos - self.data.center).length() / self.data.min_edge_len
    }
}
