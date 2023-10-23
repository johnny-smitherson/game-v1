// use std::collections::{vec_deque, VecDeque};

use bevy::prelude::*;
use rand::Rng;

use bevy::render::mesh::{Indices, PrimitiveTopology};
use rayon::prelude::IntoParallelRefMutIterator;
#[derive(Debug, Clone, Copy)]
struct TriangleData {
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
use super::height::{apply_height, height, PLANET_RADIUS};

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

use super::menu::UiState;
use crate::height::TerrainSettings;

#[allow(clippy::type_complexity)]
fn update_triangle_split(
    player_query: Query<&Transform, With<PlayerComp>>,
    mut tri_query: Query<
        (&mut Triangle, &mut Handle<Mesh>),
        (With<Triangle>, With<Handle<Mesh>>, Without<PlayerComp>),
    >,
    mut ui_state: ResMut<UiState>,

    mut meshes: ResMut<Assets<Mesh>>,
) {
    let player_pos = player_query.single().translation;
    // serial version
    // for (mut tri, mut mesh_handle) in tri_query.iter_mut() {
    //     let changed = tri.as_mut().update_split(&player_pos);
    //     if changed {
    //         // mesh.
    //         // info!("changing mesh! count={}", tri.tri_count());
    //         let new_mesh = meshes.add(tri.generate_mesh());
    //         let old_mesh = mesh_handle.clone();
    //         *mesh_handle = new_mesh;
    //         meshes.remove(old_mesh);
    //     }
    // }

    // Parallel version
    // Extract query components into a vector, since the Query<> dont work with Rayone, and
    // the Query<> .par_for_each() and .par_for_each_mut() don't work with
    let mut query_args: Vec<_> = tri_query.iter_mut().collect();

    use rayon::iter::ParallelIterator;
    let query_results: Vec<Option<_>> = query_args
        .par_iter_mut()
        .map(|(tri, mesh_handle)| {
            let changed = tri.update_split(&player_pos, &ui_state.settings);
            if changed {
                Some((mesh_handle, (tri.generate_mesh(&ui_state.settings))))
            } else {
                None
            }
        })
        .collect();
    query_results.into_iter().for_each(|result| {
        if let Some((mesh_handle, new_mesh)) = result {
            let old_mesh_handle = (*mesh_handle).clone();
            let new_mesh_handle = meshes.add(new_mesh);
            **mesh_handle = new_mesh_handle;
            meshes.remove(old_mesh_handle);
        }
    });

    let mut triangle_count = 0;
    let mut mesh_count = 0;
    for (triangle, _) in query_args {
        triangle_count += triangle.tri_count();
        mesh_count += 1;
    }

    ui_state.triangle_count = triangle_count as f32;
    ui_state.mesh_count = mesh_count as f32;
}

pub trait Piramidesc {
    fn base_tris(&self) -> Vec<Triangle>;
}

/// AM PIRAMIDĂ
#[derive(Debug, Clone)]
pub struct Piramidă<const N: usize> {
    pub children: [Triangle; N],
}

impl<const N: usize> Piramidesc for Piramidă<N> {
    fn base_tris(&self) -> Vec<Triangle> {
        let mut vec = Vec::<Triangle>::new();
        vec.extend(self.children.iter().cloned());
        vec
    }
}

impl Piramidă<1> {
    pub fn new() -> Self {
        let v1 = Vec3::new(0., 0., 1.);
        let v2 = Vec3::new(-1., 0., -1.);
        let v3 = Vec3::new(1., 0., -1.);
        Self {
            children: [
                Triangle::new([v1 * PLANET_RADIUS, v2* PLANET_RADIUS, v3* PLANET_RADIUS], 0, 0).reverse_points()
            ]
        }
    }
}

impl Piramidă<4> {
    pub fn new() -> Self {
        let v1 = Vec3::new((8.0_f32 / 9.).sqrt(), 0., -1. / 3.);
        let v2 = Vec3::new(-(2.0_f32 / 9.).sqrt(), (2.0_f32 / 3.).sqrt(), -1. / 3.);
        let v3 = Vec3::new(-(2.0_f32 / 9.).sqrt(), -(2.0_f32 / 3.).sqrt(), -1. / 3.);
        let v4 = Vec3::new(0., 0., 1.);
        Self {
            children: [
                Triangle::new([v1, v2, v3], 0, 1).reverse_points(),
                Triangle::new([v1, v3, v4], 0, 2).reverse_points(),
                Triangle::new([v2, v1, v4], 0, 3).reverse_points(),
                Triangle::new([v3, v2, v4], 0, 4).reverse_points(),
            ],
        }
    }
}

impl Piramidă<20> {
    pub fn new() -> Self {
        let v1 = Vec3::new(0., -0.525731, 0.850651);
        let v2 = Vec3::new(0.850651, 0., 0.525731);
        let v3 = Vec3::new(0.850651, 0., -0.525731);
        let v4 = Vec3::new(-0.850651, 0., -0.525731);
        let v5 = Vec3::new(-0.850651, 0., 0.525731);
        let v6 = Vec3::new(-0.525731, 0.850651, 0.);
        let v7 = Vec3::new(0.525731, 0.850651, 0.);
        let v8 = Vec3::new(0.525731, -0.850651, 0.);
        let v9 = Vec3::new(-0.525731, -0.850651, 0.);
        let v10 = Vec3::new(0., -0.525731, -0.850651);
        let v11 = Vec3::new(0., 0.525731, -0.850651);
        let v12 = Vec3::new(0., 0.525731, 0.850651);

        Self {
            children: [
                Triangle::new([v2, v3, v7], 0, 1),
                Triangle::new([v2, v8, v3], 0, 2),
                Triangle::new([v4, v5, v6], 0, 3),
                Triangle::new([v5, v4, v9], 0, 4),
                Triangle::new([v7, v6, v12], 0, 5),
                Triangle::new([v6, v7, v11], 0, 6),
                Triangle::new([v10, v11, v3], 0, 7),
                Triangle::new([v11, v10, v4], 0, 8),
                Triangle::new([v8, v9, v10], 0, 9),
                Triangle::new([v9, v8, v1], 0, 10),
                Triangle::new([v12, v1, v2], 0, 11),
                Triangle::new([v1, v12, v5], 0, 12),
                Triangle::new([v7, v3, v11], 0, 13),
                Triangle::new([v2, v7, v12], 0, 14),
                Triangle::new([v4, v6, v11], 0, 15),
                Triangle::new([v6, v5, v12], 0, 16),
                Triangle::new([v3, v8, v10], 0, 17),
                Triangle::new([v8, v2, v1], 0, 18),
                Triangle::new([v4, v10, v9], 0, 19),
                Triangle::new([v5, v9, v1], 0, 20),
            ],
        }
    }
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
pub struct PiramidăComp;

use super::player::PlayerComp;

#[derive(Component)]
pub struct CrosshairCubeX;

#[derive(Component)]
pub struct CrosshairCubeY;

#[derive(Component)]
pub struct CrosshairCubeZ;

#[derive(Component)]
pub struct Crosshair<const N: usize>;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut player_query: Query<Entity, With<PlayerComp>>,
    mut flycam_state: ResMut<InputState>,
    ui_state: ResMut<UiState>,
) {
    warn!("TRIANGLE/PYRAMID SETUP SYSTEM...");
    flycam_state.as_mut().pitch = 0.;

    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let crosshairs_id_x = commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.9, 0.1, 0.1).into()),
                transform: Transform::from_xyz(0.0, 0.0, 0.0)
                    .with_scale(Vec3::new(0.03, 0.03, 0.03)),
                ..default()
            },
            CrosshairCubeX,
        ))
        .id();
    let crosshairs_id_y = commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.1, 0.9, 0.1).into()),
                transform: Transform::from_xyz(0.0, 0.0, 0.0)
                    .with_scale(Vec3::new(0.03, 0.03, 0.03)),
                ..default()
            },
            CrosshairCubeY,
        ))
        .id();
    let crosshairs_id_z = commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.1, 0.1, 0.9).into()),
                transform: Transform::from_xyz(0.0, 0.0, 0.0)
                    .with_scale(Vec3::new(0.03, 0.03, 0.03)),
                ..default()
            },
            CrosshairCubeZ,
        ))
        .id();


    let piramidă = Box::new(Piramidă::<1>::new());

    let tris = piramidă.as_ref().base_tris();
    let planet_ent = commands
        .spawn((
            PiramidăComp,
            SpatialBundle {
                transform: Transform::from_xyz(
                    0.,
                    0.0,
                    0.0,
                ), // .with_rotation(Quat::from_rotation_x(-PI / 4.)),
                ..default()
            },
        ))
        .id();

        let player = player_query.get_single_mut().expect("player not found!!");
        commands.entity(player).set_parent(planet_ent);
        commands.entity(crosshairs_id_x).set_parent(planet_ent);
        commands.entity(crosshairs_id_y).set_parent(planet_ent);
        commands.entity(crosshairs_id_z).set_parent(planet_ent);
    for (_tri_idx, tri) in tris.into_iter().enumerate() {
        let tri_ent = commands
            .spawn((
                PbrBundle {
                    mesh: meshes.add(tri.generate_mesh(&ui_state.settings)),
                    material: debug_material.clone(),
                    ..default()
                },
                tri,
                // PickableBundle::default(),
            ))
            .id();
        commands.entity(tri_ent).set_parent(planet_ent);
    }
}

fn rotate_planet(mut query_piramidă: Query<&mut Transform, With<PiramidăComp>>, time: Res<Time>) {
    for mut transform in &mut query_piramidă {
        transform.rotate_x(time.delta_seconds() / 2.);
    }
}

// use bevy::ecs::event::Events;
use crate::player::{FlyCam, InputState, MovementSettings};
use bevy::input::mouse::MouseMotion;
use bevy::input::mouse::MouseWheel;
use bevy::window::{CursorGrabMode, PrimaryWindow};

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn rotate_player(
    mut query_player: Query<(&mut Transform, &mut PlayerComp), With<PlayerComp>>,
    mut query_camera: Query<&mut Transform, (With<FlyCam>, Without<PlayerComp>)>,

    mut query_crosshair_x: Query<
        (&mut Transform, Entity),
        (With<CrosshairCubeX>, Without<FlyCam>, Without<PlayerComp>),
    >,

    mut query_crosshair_y: Query<
        &mut Transform,
        (
            With<CrosshairCubeY>,
            Without<CrosshairCubeX>,
            Without<FlyCam>,
            Without<PlayerComp>,
        ),
    >,

    mut query_crosshair_z: Query<
        &mut Transform,
        (
            With<CrosshairCubeZ>,
            Without<CrosshairCubeX>,
            Without<CrosshairCubeY>,
            Without<FlyCam>,
            Without<PlayerComp>,
        ),
    >,
    time: Res<Time>,
    mut flycam_state: ResMut<InputState>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mouse_motion_events: Res<Events<MouseMotion>>,
    mut scroll_evr: EventReader<MouseWheel>,
    keys: Res<Input<KeyCode>>,
    settings: Res<MovementSettings>,
    ui_state: Res<UiState>,
) {
    // mouse movements
    let delta_state = flycam_state.as_mut();
    let mut mouse_delta_pitch = delta_state.pitch;
    let mut mouse_delta_yaw = 0.0;

    let window = primary_window.get_single_mut().expect("no window wtf");
    let is_mouse_grabbed = window.cursor.grab_mode != CursorGrabMode::None;
    // capture mouse motion events
    for ev in delta_state.reader_motion.iter(&mouse_motion_events) {
        if is_mouse_grabbed {
            // Using smallest of height or width ensures equal vertical and horizontal sensitivity
            let window_scale = window.height().min(window.width());
            mouse_delta_pitch -= (settings.sensitivity * ev.delta.y * window_scale).to_radians();
            mouse_delta_pitch = mouse_delta_pitch.clamp(-1.54, 1.54);
            delta_state.pitch = mouse_delta_pitch;
            mouse_delta_yaw -= (settings.sensitivity * ev.delta.x * window_scale).to_radians();
        }
    }

    let (mut player_tr, mut player_comp) = query_player.get_single_mut().expect("no player");
    let mut camera_tr = query_camera.get_single_mut().expect("no camera");

    let (mut crosshair_x, _entity) = query_crosshair_x
        .get_single_mut()
        .expect("no crosshair cube");
    let mut crosshair_y = query_crosshair_y
        .get_single_mut()
        .expect("no crosshair cube");
    let mut crosshair_z = query_crosshair_z
        .get_single_mut()
        .expect("no crosshair cube");

    // capture movement events: wasd/scroll wheel
    {
        let mut velocity = Vec3::ZERO;
        let _up = Vec3::Y;
        let _right = player_tr.right().normalize();
        let _forward = _up.cross(_right).normalize();

        for key in keys.get_pressed() {
            if is_mouse_grabbed {
                match key {
                    KeyCode::W => velocity += _forward,
                    KeyCode::S => velocity -= _forward,
                    KeyCode::A => velocity -= _right,
                    KeyCode::D => velocity += _right,
                    // KeyCode::Space => velocity += _up,
                    // KeyCode::ControlLeft => velocity -= _up,
                    _ => (),
                }
            }
        }

        if !is_mouse_grabbed && ui_state.enable_animation {
            // println!("{}", time.elapsed_seconds());
            velocity += _forward * 1.0; //(time.elapsed_seconds() / 3.0 + 1.0).sin();
            velocity += _right * (time.elapsed_seconds() / 4.0 + 2.0).cos();
            velocity += _up * (time.elapsed_seconds() / 5.0 + 3.0).sin();
            mouse_delta_yaw += (time.elapsed_seconds() / 8.0 + 2.0).sin() / 100.0;

            // copmute camera height anim with exp
            let camera_min_exp = ui_state.settings.MIN_CAMERA_HEIGHT.log2();
            let camera_max_exp = ui_state.settings.MAX_CAMERA_HEIGHT.log2();
            let camera_osc = ((time.elapsed_seconds() / 6.0).cos() + 1.) / 2.0;
            let current_camera_exp =
                camera_min_exp + (camera_max_exp - camera_min_exp) * camera_osc;
            let new_camera_height = 2.0_f32.powf(current_camera_exp);
            player_comp.camera_height = new_camera_height;

            delta_state.pitch = -1.5 * current_camera_exp / camera_max_exp;
        }

        {
            use bevy::input::mouse::MouseScrollUnit;
            for ev in scroll_evr.iter() {
                match ev.unit {
                    MouseScrollUnit::Line => {
                        player_comp.camera_height /= (ev.y.clamp(-1.0, 1.0) + 10.0) / 10.0;
                    }
                    MouseScrollUnit::Pixel => {
                        player_comp.camera_height /= (ev.y.clamp(-1.0, 1.0) + 10.0) / 10.0;
                    }
                }
            }
            player_comp.camera_height = player_comp.camera_height.clamp(
                ui_state.settings.MIN_CAMERA_HEIGHT,
                ui_state.settings.MAX_CAMERA_HEIGHT,
            );
        }

        velocity = velocity.normalize_or_zero();

        player_tr.translation +=
            velocity * time.delta_seconds() * settings.speed * player_comp.camera_height;
    }

    camera_tr.rotation = Quat::from_axis_angle(Vec3::X, delta_state.pitch);
    player_tr.rotate_local_y(mouse_delta_yaw);

    let _up = Vec3::Y;
    let _right = player_tr.right().normalize();
    let _forward = _up.cross(_right).normalize();

    player_tr.translation.y = player_comp.camera_height + height(& player_tr.translation);
    let mut _pos_xz = Vec3::new(player_tr.translation.x, 0.0, player_tr.translation.z);
    let max_camera_xz = PLANET_RADIUS / 5.0;
    if _pos_xz.length() > max_camera_xz {
        _pos_xz = _pos_xz.normalize() * max_camera_xz;
        player_tr.translation.x = _pos_xz.x;
        player_tr.translation.z = _pos_xz.z;
    }
    let _target = player_tr.translation + _forward;
    player_tr.look_at(_target, _up);

    crosshair_z.translation = player_tr.translation + _forward;
    crosshair_y.translation = player_tr.translation + _up;
    crosshair_x.translation = player_tr.translation + _right;
}

pub struct PiramidePlugin;
impl Plugin for PiramidePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, setup)
            .add_systems(Update, (rotate_planet, rotate_player, update_triangle_split).chain());
    }
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    use bevy::{
        // core_pipeline::bloom::BloomSettings,
        prelude::*,
        render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    };
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}
