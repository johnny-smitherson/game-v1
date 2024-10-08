use std::time::Duration;

// use std::collections::{vec_deque, VecDeque};
use super::menu::UiMenuState;
use crate::piramida::Piramidesc;
use crate::piramida::Piramidă;
use crate::raycast::TerrainRaycastSet;
use crate::triangle::Triangle;

use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy_mod_raycast::RaycastMesh;
use bevy_rapier3d::prelude::*;
use bevy_spatial::kdtree::KDTree3;
use bevy_spatial::AutomaticUpdate;
use bevy_spatial::SpatialAccess;
use bevy_spatial::TransformMode;
use rayon::prelude::IntoParallelRefMutIterator;

pub struct PlanetPlugin;
impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Triangle>()
            .add_systems(Startup, setup_planet)
            .add_systems(PostUpdate, update_triangle_split)
            .add_plugins(
                AutomaticUpdate::<TerrainSplitProbe>::new()
                    .with_frequency(Duration::from_secs_f32(1.0 / 60.0))
                    .with_transform(TransformMode::GlobalTransform),
            );
    }
}

#[derive(Reflect, Component, Default)]
pub struct TerrainSplitProbe;

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
pub struct PlanetComponent;

#[allow(clippy::type_complexity)]
fn update_triangle_split(
    // probe_query: Query<&Transform, With<TerrainSplitProbe>>,
    probe_tree: Res<KDTree3<TerrainSplitProbe>>,
    mut tri_query: Query<
        (&mut Triangle, &mut Handle<Mesh>, &mut Collider, &mut Aabb),
        (
            With<Triangle>,
            With<Handle<Mesh>>,
            With<Collider>,
            Without<TerrainSplitProbe>,
        ),
    >,
    mut ui_state: ResMut<UiMenuState>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // let probe_pos: Vec<Vec3> = probe_query.iter().map(|x| x.translation).collect();
    // let probe_dist = |pos: &Vec3| -> f32 {
    //     if probe_pos.is_empty() {
    //         return 666666.0_f32;
    //     }
    //     probe_pos
    //         .iter()
    //         .map(|pp| (*pp - *pos).length())
    //         .min_by(|a, b| a.partial_cmp(b).unwrap())
    //         .unwrap()
    // };

    let probe_dist_from_tree = |pos: &Vec3| -> f32 {
        if let Some(nn) = probe_tree.nearest_neighbour(*pos) {
            nn.0.distance(*pos)
        } else {
            666666.0_f32
        }
    };
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
        .map(|(tri, mesh_handle, collider, aabb)| {
            let changed = tri.update_split(&probe_dist_from_tree, &ui_state.settings);
            if changed {
                let (mesh, new_collider) = tri.generate_mesh(&ui_state.settings);
                *collider.as_mut() = new_collider;
                **aabb = mesh.compute_aabb().expect("tri mesh returned empty aabb");
                Some((mesh_handle, mesh))
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
    for (triangle, _, _, _) in query_args {
        triangle_count += triangle.tri_count();
        mesh_count += 1;
    }

    ui_state.triangle_count = triangle_count as f32;
    ui_state.mesh_count = mesh_count as f32;
}

fn setup_planet(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ui_state: ResMut<UiMenuState>,
) {
    warn!("TRIANGLE/PYRAMID SETUP SYSTEM...");

    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let mut piramidă = Box::new(Piramidă::<1>::new());

    let tris = piramidă.as_mut().base_tris();
    let planet_ent = commands
        .spawn((
            PlanetComponent,
            SpatialBundle::default(),
            Name::new("THE PLANET"),
        ))
        .id();

    for (_tri_idx, tri) in tris.into_iter().enumerate() {
        let (mesh, collider) = tri.generate_mesh(&ui_state.settings);
        let mesh_asset = meshes.add(mesh);
        let mut name = "Base Planet Triangle ".to_owned();
        name.push_str(tri.coord());

        let tri_ent = commands
            .spawn((
                PbrBundle {
                    mesh: mesh_asset,
                    material: debug_material.clone(),
                    ..default()
                },
                tri,
                RigidBody::Fixed,
                collider,
                Ccd::enabled(),
                // PickableBundle::default(),
                RaycastMesh::<TerrainRaycastSet>::default(),
                Name::new(name),
            ))
            .id();
        commands.entity(tri_ent).set_parent(planet_ent);
    }
}

// Creates a colorful test pattern
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
