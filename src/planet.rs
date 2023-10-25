// use std::collections::{vec_deque, VecDeque};
use super::height::{height, PLANET_RADIUS};
use super::menu::UiMenuState;
use crate::game_assets::{BulletAssets, GameSceneAssets};
use crate::piramida::Piramidesc;
use crate::piramida::Piramidă;
use crate::triangle::Triangle;
use bevy::prelude::shape::Cube;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rayon::prelude::IntoParallelRefMutIterator;

pub struct PlanetPlugin;
impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_planet)
            .add_systems(Update, (rotate_player, update_triangle_split).chain())
            .add_systems(Update, (shoot_bullet, capture_bullet_impact).chain())
            .add_systems(PostUpdate, on_bullet_impact);
    }
}

#[derive(Component, Default)]
pub struct Bullet;

#[derive(Component)]
pub struct BulletHit {
    other_thing_hit: Entity,
    // hit_location: Vec3,
}

fn spawn_gltf(mut commands: Commands, scene_assets: Mut<GameSceneAssets>) {
    // note that we have to include the `Scene0` label

    // to position our 3d model, simply use the Transform
    // in the SceneBundle
    commands.spawn(SceneBundle {
        scene: scene_assets
            .scenes
            .get("ORIGINAL/Tanks and Armored Vehicle.glb")
            .expect("KEY NOT FOUND")
            .clone(),
        transform: Transform::from_xyz(2.0, 0.0, -5.0),
        ..Default::default()
    });
}

fn on_bullet_impact(
    mut commands: Commands,
    mut hits: Query<(Entity, &mut Bullet, &mut BulletHit)>,
) {
    for (bullet_ent, mut bullet, bullet_hit) in hits.iter() {
        // trigger some events and shit
        // TODO

        // finally, delete the bullet
        commands.entity(bullet_ent).despawn();
    }
}

fn shoot_bullet(
    mut commands: Commands,
    ui_state: Res<UiMenuState>,
    mouse_button: Res<Input<MouseButton>>,
    player_query: Query<&mut Transform, With<PlayerComponent>>,
    camera_query: Query<&mut GlobalTransform, (With<FlyCam>, Without<PlayerComponent>)>,
    bullet_assets: Res<BulletAssets>,
) {
    if !ui_state.is_mouse_captured {
        return;
    }
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    let camera_tr = camera_query.get_single().expect("no camera wtf.");
    let player_tr = player_query.get_single().expect("no player wtf.");

    const SHOOT_IMPULSE: f32 = 100.0;
    const SHOOT_ROTATION: f32 = 10.0;
    const SHOOT_EXTRA_FORWARD: f32 = 1.5;

    let fwd = camera_tr.forward();
    let quat = Quat::from_rotation_arc(Vec3::Z, fwd);
    let spawn_pos = player_tr.translation + fwd * SHOOT_EXTRA_FORWARD;

    let bullet_id = commands
        .spawn((
            Bullet,
            PbrBundle {
                mesh: bullet_assets.mesh.clone(),
                material: bullet_assets.material.clone(),
                transform: Transform::from_translation(spawn_pos).with_rotation(quat),
                ..default()
            },
        ))
        .insert(RigidBody::Dynamic)
        .insert(ColliderMassProperties::Density(2.0))
        .insert(bullet_assets.collider.clone())
        .insert(Ccd::enabled())
        .insert(Damping {
            linear_damping: 0.05,
            angular_damping: 0.05,
        })
        .insert(ExternalImpulse {
            impulse: fwd * SHOOT_IMPULSE,
            torque_impulse: quat * Vec3::new(0.0, 0.0, SHOOT_ROTATION),
        })
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Name::new("BULLET"))
        .id();

    commands
        .spawn(bevy_hanabi::prelude::ParticleEffectBundle {
            effect: bevy_hanabi::ParticleEffect::new(bullet_assets.flying_effect.clone()),
            ..Default::default()
        })
        .set_parent(bullet_id);
}

fn capture_bullet_impact(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut bullet_query: Query<Entity, With<Bullet>>,
) {
    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(col1, col2, _flags) = collision_event {
            if bullet_query.contains(col1.clone()) {
                commands.entity(col1.clone()).insert(BulletHit {
                    other_thing_hit: col2.clone(),
                });
            }
            if bullet_query.contains(col2.clone()) {
                commands.entity(col2.clone()).insert(BulletHit {
                    other_thing_hit: col1.clone(),
                });
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn update_triangle_split(
    player_query: Query<&Transform, With<PlayerComponent>>,
    mut tri_query: Query<
        (&mut Triangle, &mut Handle<Mesh>, &mut Collider),
        (
            With<Triangle>,
            With<Handle<Mesh>>,
            With<Collider>,
            Without<PlayerComponent>,
        ),
    >,
    mut ui_state: ResMut<UiMenuState>,
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
        .map(|(tri, mesh_handle, collider)| {
            let changed = tri.update_split(&player_pos, &ui_state.settings);
            if changed {
                let (mesh, new_collider) = tri.generate_mesh(&ui_state.settings);
                *collider.as_mut() = new_collider;
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
    for (triangle, _, _) in query_args {
        triangle_count += triangle.tri_count();
        mesh_count += 1;
    }

    ui_state.triangle_count = triangle_count as f32;
    ui_state.mesh_count = mesh_count as f32;
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
pub struct PlanetComponent;

use super::player::PlayerComponent;

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

    let piramidă = Box::new(Piramidă::<1>::new());

    let tris = piramidă.as_ref().base_tris();
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
                Name::new("some base planet triangle"),
            ))
            .id();
        commands.entity(tri_ent).set_parent(planet_ent);
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
    mut query_player: Query<(&mut Transform, &mut PlayerComponent), With<PlayerComponent>>,
    mut query_camera: Query<&mut Transform, (With<FlyCam>, Without<PlayerComponent>)>,

    time: Res<Time>,
    mut mouse_input_state: ResMut<InputState>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mouse_motion_events: Res<Events<MouseMotion>>,
    mut scroll_evr: EventReader<MouseWheel>,
    keys: Res<Input<KeyCode>>,
    settings: Res<MovementSettings>,
    mut ui_state: ResMut<UiMenuState>,
) {
    // mouse movements
    let delta_state = mouse_input_state.as_mut();
    let mut mouse_delta_pitch = delta_state.pitch;
    let mut mouse_delta_yaw = 0.0;

    let window = primary_window.get_single_mut().expect("no window wtf");
    ui_state.is_mouse_captured = window.cursor.grab_mode != CursorGrabMode::None;
    // capture mouse motion events
    for ev in delta_state.reader_motion.iter(&mouse_motion_events) {
        if ui_state.is_mouse_captured {
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

    // capture movement events: wasd/scroll wheel
    {
        let mut velocity = Vec3::ZERO;
        let _up = Vec3::Y;
        let _right = player_tr.right().normalize();
        let _forward = _up.cross(_right).normalize();

        for key in keys.get_pressed() {
            if ui_state.is_mouse_captured {
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

        if !ui_state.is_mouse_captured && ui_state.enable_animation {
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
                if ui_state.is_mouse_captured {
                    match ev.unit {
                        MouseScrollUnit::Line => {
                            player_comp.camera_height /= (ev.y.clamp(-1.0, 1.0) + 10.0) / 10.0;
                        }
                        MouseScrollUnit::Pixel => {
                            player_comp.camera_height /= (ev.y.clamp(-1.0, 1.0) + 10.0) / 10.0;
                        }
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

    player_tr.translation.y = player_comp.camera_height + height(&player_tr.translation);
    let mut _pos_xz = Vec3::new(player_tr.translation.x, 0.0, player_tr.translation.z);
    let max_camera_xz = PLANET_RADIUS / 5.0;
    if _pos_xz.length() > max_camera_xz {
        _pos_xz = _pos_xz.normalize() * max_camera_xz;
        player_tr.translation.x = _pos_xz.x;
        player_tr.translation.z = _pos_xz.z;
    }
    let _target = player_tr.translation + _forward;
    player_tr.look_at(_target, _up);
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
