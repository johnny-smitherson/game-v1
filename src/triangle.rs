// use std::collections::{vec_deque, VecDeque};

use bevy::prelude::*;
use rand::Rng;

use bevy::render::mesh::{Indices, PrimitiveTopology};
// use rayon::prelude::IntoParallelRefMutIterator;

use super::terrain::apply_height;
use crate::terrain::{TerrainSettings, BASE_SPLIT_LEVEL};
use bevy_rapier3d::prelude::*;

#[derive(Reflect, Debug, Clone, Copy)]
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
#[derive(Reflect, Component, Debug, Clone)]
pub struct Triangle {
    /// info for current triangle
    data: TriangleData,
    /// info for current triangle if no children, else all child triangles. used to build mesh into
    all_data: Vec<TriangleData>,
    /// info for building skirts around this triangle. Generated and kept all the time.
    skirt_data: [TriangleData; 3],
    /// info for the required skirts for all sub-nodes.
    all_skirt_data: Vec<TriangleData>,
    /// depth of current node
    pub level: u8,
    /// index in the parent's list of children
    pub id: u8,
    /// list of children
    #[reflect(ignore)]
    pub children: Option<[Box<Triangle>; 4]>,
    /// max level of this sub-tree where we find leafs
    pub max_leaf_level: u8,
    /// min level of this sub-tree where we find leafs
    pub min_leaf_level: u8,
    /// node key (chain of IDs)
    coord: String,
    /// marks if this is first update happened or not
    was_updated: bool,
}

impl Default for Triangle {
    fn default() -> Self {
        Triangle::new(
            [
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, -0.5, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            ],
            0,
            0,
            "",
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
    pub fn new(points: [Vec3; 3], level: u8, id: u8, parent_coord: &str) -> Self {
        let data = TriangleData::new([
            apply_height(&points[0]),
            apply_height(&points[1]),
            apply_height(&points[2]),
        ]);

        let coord = {
            let mut coord = parent_coord.to_owned();
            if !coord.is_empty() {
                coord.push('.');
            }
            coord.push_str(id.to_string().as_str());
            coord
        };

        let skirt_data = {
            let [v1, v2, v3] = data.verts;
            let v12 = (v1 + v2) * 0.5;
            let v23 = (v2 + v3) * 0.5;
            let v13 = (v1 + v3) * 0.5;
            [
                TriangleData::new([v1, apply_height(&v12), v2]),
                TriangleData::new([v2, apply_height(&v23), v3]),
                TriangleData::new([v3, apply_height(&v13), v1]),
            ]
        };

        Self {
            data,
            all_data: vec![data],
            all_skirt_data: vec![],
            level,
            id,
            children: None,
            max_leaf_level: level,
            min_leaf_level: level,
            coord,
            skirt_data,
            was_updated: false,
        }
    }
    pub fn coord(&self) -> &str {
        return self.coord.as_str();
    }

    pub fn reverse_points(&self) -> Self {
        Self::new(
            [self.data.verts[1], self.data.verts[0], self.data.verts[2]],
            self.level,
            self.id,
            "",
        )
    }

    pub fn base_tris(&mut self) -> Vec<Triangle> {
        if self.level == BASE_SPLIT_LEVEL {
            return vec![self.clone()];
        }
        assert!(self.level < BASE_SPLIT_LEVEL);
        self.split();

        let mut vec = Vec::<Triangle>::new();
        for child in self
            .children
            .as_mut()
            .expect("no children after split?!")
            .iter_mut()
        {
            vec.extend(child.base_tris());
        }
        vec
    }

    pub fn generate_mesh(&self, _settings: &TerrainSettings) -> (Mesh, Collider) {
        assert!(self.level == BASE_SPLIT_LEVEL);

        let mut all_verts = Vec::<Vec3>::new();
        let mut all_norms = Vec::<Vec3>::new();
        let mut all_uvs = Vec::<Vec2>::new();
        let mut all_indices = Vec::<u32>::new();
        let mut all_indices_grp = Vec::<[u32; 3]>::new();

        let mut idx: u32 = 0;
        for data in self.all_data.iter().chain(self.all_skirt_data.iter()) {
            all_verts.extend_from_slice(&data.verts);
            all_norms.extend_from_slice(&data.norm);
            all_uvs.extend_from_slice(&data.uvs);
            all_indices.extend_from_slice(&[idx, idx + 1, idx + 2]);
            all_indices_grp.push([idx, idx + 1, idx + 2]);
            idx += 3;
        }
        let collider = Collider::trimesh(all_verts.clone(), all_indices_grp);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(all_indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, all_verts);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, all_norms);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, all_uvs);

        // let collider =  Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh).expect("mesh incompatible with physics lib");
        (mesh, collider)
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

        /*
        Triangle ID vs. vertex ID.
        ->            v1
        ->            /\
        ->           /  \
        ->          / 2  \
        ->      v12 ------ v13
        ->        / \ 1  / \
        ->       /   \  /   \
        ->      /  3  \/  4  \
        ->      ---------------
        ->    v2      v23       v3
        id=1 is neighbour of 2,3,4.
        */
        self.children = Some([
            Box::new(Triangle::new(
                [v12, v23, v13],
                self.level + 1,
                1,
                self.coord(),
            )),
            Box::new(Triangle::new(
                [v1, v12, v13],
                self.level + 1,
                2,
                self.coord(),
            )),
            Box::new(Triangle::new(
                [v12, v2, v23],
                self.level + 1,
                3,
                self.coord(),
            )),
            Box::new(Triangle::new(
                [v13, v23, v3],
                self.level + 1,
                4,
                self.coord(),
            )),
        ]);
        self.max_leaf_level = self.level + 1;
    }
    fn merge(&mut self) {
        assert!(self.is_split(), "can't mrege without children");
        self.children = None;
        self.max_leaf_level = self.level;
    }

    pub fn update_split(&mut self, pos: &Vec<Vec3>, settings: &TerrainSettings) -> bool {
        let dirty = self._do_update_split(pos, settings);

        if !self.was_updated || dirty {
            self.was_updated = true;
        }
        dirty
    }

    /// returns true if we changed something notable and you wanna update the thing
    fn _do_update_split(&mut self, pos: &Vec<Vec3>, settings: &TerrainSettings) -> bool {
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
            let child_results: Vec<_> = self
                .children
                .as_mut()
                .expect("wtf")
                .par_iter_mut()
                .map(|child| child.as_mut()._do_update_split(pos, settings))
                .collect();

            for child_dirty in child_results {
                dirty = child_dirty || dirty;
            }
        }
        // pull data from children
        if ((!self.was_updated) || dirty) && self.level >= BASE_SPLIT_LEVEL {
            self.all_data.clear();
            self.all_skirt_data.clear();
            if self.is_split() {
                self.max_leaf_level = self
                    .children
                    .as_ref()
                    .expect("no children")
                    .iter()
                    .map(|x| x.max_leaf_level)
                    .max()
                    .expect("no children");
                self.min_leaf_level = self
                    .children
                    .as_ref()
                    .expect("no children")
                    .iter()
                    .map(|x| x.min_leaf_level)
                    .min()
                    .expect("no children");

                for child in self.children.as_mut().expect("wtf").iter_mut() {
                    self.all_data.extend(child.all_data.iter());

                    // if child is leaf but non-max-level, add its skirts to itself
                    if (!child.is_split())
                        && (child.level < self.max_leaf_level || child.level == self.min_leaf_level)
                    {
                        for t in child.skirt_data.iter() {
                            // in skirts, the midpoint is always the second point. we only want the skirt if the point is lower
                            if t.verts[1].y <= (t.verts[0].y + t.verts[2].y) * 0.5 {
                                child.all_skirt_data.push(*t);
                            }
                        }
                    }

                    self.all_skirt_data.extend(child.all_skirt_data.iter());
                }
            } else {
                self.all_data.push(self.data);
                self.max_leaf_level = self.level;
                self.min_leaf_level = self.level;
            }
        }
        dirty
    }

    pub fn tri_count(&self) -> usize {
        self.all_data.len()

        // if !self.is_split() {
        //     1
        // } else {
        //     let mut count: usize = 0;
        //     for child in self.children.as_ref().expect("msg").iter().as_ref() {
        //         count += child.tri_count();
        //     }
        //     count
        // }
    }

    fn should_split(&self, pos: &Vec<Vec3>, settings: &TerrainSettings) -> bool {
        if self.level < settings.MIN_SPLIT_LEVEL || self.level < BASE_SPLIT_LEVEL {
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

    fn should_merge(&self, pos: &Vec<Vec3>, settings: &TerrainSettings) -> bool {
        if self.level <= settings.MIN_SPLIT_LEVEL || self.level <= BASE_SPLIT_LEVEL {
            return false;
        }

        if self.level > settings.MAX_SPLIT_LEVEL {
            true
        } else {
            self.get_distance_over_size(pos)
                > (1.0 + settings.SPLIT_LAZY_COEF) * settings.TESSELATION_VALUE
        }
    }

    fn get_distance_over_size(&self, all_pos: &Vec<Vec3>) -> f32 {
        if all_pos.is_empty() {
            return 666666.0_f32;
        }

        all_pos
            .iter()
            .map(|pos| (*pos - self.data.center).length())
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
            / self.data.min_edge_len
    }
}
