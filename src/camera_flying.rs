use bevy::ecs::event::ManualEventReader;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy_atmosphere::prelude::*;
use bevy_inspector_egui::prelude::InspectorOptions;
use bevy_mod_raycast::RaycastSource;
use core::f32::consts::PI;
use smart_default::SmartDefault;

use bevy::core_pipeline::bloom::BloomSettings;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::window::{CursorGrabMode, PrimaryWindow};

use bevy::{
    core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
    pbr::ScreenSpaceAmbientOcclusionBundle,
};

use crate::audio::SpatialAudioListener;
use crate::planet::TerrainSplitProbe;
use crate::raycast::TerrainRaycastSet;
use crate::terrain::PLANET_MAX_PLAY_RADIUS;

// use std::collections::{vec_deque, VecDeque};
use super::menu::UiMenuState;
use super::terrain::height;

// use bevy::ecs::event::Events;
use bevy::input::mouse::MouseWheel;

pub struct FlyingCameraPlugin;
impl Plugin for FlyingCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TemporalAntiAliasPlugin)
            .init_resource::<FlyingCameraInputState>()
            .register_type::<FlyingCameraInputState>()
            .init_resource::<FlyingCameraMovementSettings>()
            .register_type::<FlyingCameraMovementSettings>()
            .add_plugins(AtmospherePlugin)
            .insert_resource(AtmosphereModel::default())
            .insert_resource(SunSettings::default())
            .register_type::<SunSettings>()
            .insert_resource(SunCycleTimer(Timer::new(
                bevy::utils::Duration::from_millis(500),
                TimerMode::Repeating,
            )))
            .add_systems(PreStartup, (setup_flying_camera, setup_sun))
            .add_systems(
                Update,
                (cursor_grab, daylight_cycle, rotate_camera_by_mouse).chain(),
            );
    }
}

#[derive(Bundle, Default)]
pub struct FlyingCameraBundle {
    pub spatial: SpatialBundle,
    pub camera_pivot: FlyingCameraPivot,
}

#[derive(Component)]
pub struct FlyingCameraPivot {
    pub camera_height: f32,
}

impl Default for FlyingCameraPivot {
    fn default() -> Self {
        Self {
            camera_height: 10.0,
        }
    }
}

/// A marker component used in queries when you want flycams and not other cameras
#[derive(Reflect, Component)]
pub struct FlyingCamera;

#[derive(Reflect, Component)]
struct Sun;

#[derive(Reflect, Resource, SmartDefault, InspectorOptions)]
#[reflect(Resource)]
struct SunSettings {
    #[inspector(min = 50.0, max = 15000.0)]
    #[default = 8000.0]
    illuminance: f32,
}

// Timer for updating the daylight cycle (updating the atmosphere every frame is slow, so it's better to do incremental changes)
#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource)]
struct SunCycleTimer(Timer);

fn daylight_cycle(
    mut atmosphere: AtmosphereMut<Nishita>,
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut timer: ResMut<SunCycleTimer>,
    time: Res<Time>,
    sun_settings: Res<SunSettings>,
) {
    timer.0.tick(time.delta());

    if timer.0.finished() {
        let time = 1.0 + time.elapsed_seconds_wrapped() / 200.0;
        let t = PI / 2.0 + time.sin() * 0.35;

        atmosphere.sun_position = Vec3::new(0., t.sin(), t.cos());

        for (mut light_trans, mut directional) in query.iter_mut() {
            light_trans.rotation = Quat::from_rotation_x(-t);
            directional.illuminance = t.sin().max(0.0).powf(2.0) * sun_settings.illuminance;
        }
    }
}

fn setup_sun(mut commands: Commands, sun_settings: Res<SunSettings>) {
    // Our Sun
    commands.spawn((
        DirectionalLightBundle {
            transform: Transform::from_rotation(Quat::from_rotation_x(-PI / 2.0)),
            directional_light: DirectionalLight {
                illuminance: sun_settings.illuminance,
                ..default()
            },
            ..Default::default()
        },
        Sun,
        Name::new("THE SUN"),
    ));
}

fn setup_flying_camera(mut commands: Commands) {
    let camera = commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    hdr: true,
                    ..default()
                },
                projection: Projection::Perspective(PerspectiveProjection {
                    near: 0.1,
                    far: 100000.0,
                    ..default()
                }),

                tonemapping: Tonemapping::BlenderFilmic,
                ..Default::default()
            },
            FlyingCamera,
            AtmosphereCamera::default(),
            RaycastSource::<TerrainRaycastSet>::new(),
            FogSettings {
                color: Color::rgba(0.1, 0.2, 0.4, 1.0),
                directional_light_color: Color::rgba(1.0, 0.95, 0.75, 0.5),
                directional_light_exponent: 30.0,
                falloff: FogFalloff::from_visibility_colors(
                    25000.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
                    Color::rgb(0.35, 0.5, 0.66), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
                    Color::rgb(0.8, 0.844, 1.0), // atmospheric inscattering color (light gained due to scattering from the sun)
                ),
            },
            BloomSettings {
                ..default() // intensity: 0.02,
                            // scale: 0.5,
                            // knee: -3.23,
                            // threshold: 0.7,
            },
            // PickingCameraBundle::default();
            Name::new("THE CAMERA"),
            SpatialAudioListener,
        ))
        .insert(ScreenSpaceAmbientOcclusionBundle::default())
        .insert(TemporalAntiAliasBundle::default())
        .id();

    let player = commands
        .spawn((
            FlyingCameraBundle { ..default() },
            TerrainSplitProbe,
            Name::new("THE FLYING CAMERA"),
        ))
        .id();
    commands.entity(camera).set_parent(player);
}

fn cursor_grab(
    keys: Res<Input<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            toggle_grab_cursor(&mut window);
        }
    } else {
        warn!("Primary window not found for `cursor_grab`!");
    }
}

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    match window.cursor.grab_mode {
        CursorGrabMode::None => {
            window.cursor.grab_mode = CursorGrabMode::Confined;
            window.cursor.visible = false;
        }
        _ => {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
    }
}

/// Keeps track of mouse motion events, pitch, and yaw
#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource)]
#[reflect(from_reflect = false)]
pub struct FlyingCameraInputState {
    pub pitch: f32,
    pub yaw: f32,
    #[reflect(ignore)]
    pub reader_motion: ManualEventReader<MouseMotion>,
}

/// Mouse sensitivity and movement speed

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource)]
pub struct FlyingCameraMovementSettings {
    pub sensitivity: f32,
    pub speed: f32,
}

impl Default for FlyingCameraMovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 4.02,
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn rotate_camera_by_mouse(
    mut pivot_query: Query<(&mut Transform, &mut FlyingCameraPivot), With<FlyingCameraPivot>>,
    mut camera_query: Query<&mut Transform, (With<FlyingCamera>, Without<FlyingCameraPivot>)>,

    time: Res<Time>,
    mut mouse_input_state: ResMut<FlyingCameraInputState>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mouse_motion_events: Res<Events<MouseMotion>>,
    mut scroll_evr: EventReader<MouseWheel>,
    keys: Res<Input<KeyCode>>,
    settings: Res<FlyingCameraMovementSettings>,
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

    let (mut pivot_tr, mut pivot_comp) = pivot_query.get_single_mut().expect("no player");
    let mut camera_tr = camera_query.get_single_mut().expect("no camera");

    // capture movement events: wasd/scroll wheel
    {
        let mut velocity = Vec3::ZERO;
        let _up = Vec3::Y;
        let _right = pivot_tr.right().normalize();
        let _forward = _up.cross(_right).normalize();

        for key in keys.get_pressed() {
            match key {
                KeyCode::W => velocity += _forward,
                KeyCode::S => velocity -= _forward,
                KeyCode::A => velocity -= _right,
                KeyCode::D => velocity += _right,
                _ => (),
            }
        }

        if ui_state.enable_animation {
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
            pivot_comp.camera_height = new_camera_height;

            delta_state.pitch = -1.5 * current_camera_exp / camera_max_exp;
        }

        {
            use bevy::input::mouse::MouseScrollUnit;
            for ev in scroll_evr.iter() {
                match ev.unit {
                    MouseScrollUnit::Line => {
                        pivot_comp.camera_height /= (ev.y.clamp(-1.0, 1.0) + 10.0) / 10.0;
                    }
                    MouseScrollUnit::Pixel => {
                        pivot_comp.camera_height /= (ev.y.clamp(-1.0, 1.0) + 10.0) / 10.0;
                    }
                }
            }
            pivot_comp.camera_height = pivot_comp.camera_height.clamp(
                ui_state.settings.MIN_CAMERA_HEIGHT,
                ui_state.settings.MAX_CAMERA_HEIGHT,
            );
        }

        velocity = velocity.normalize_or_zero();

        pivot_tr.translation +=
            velocity * time.delta_seconds() * settings.speed * pivot_comp.camera_height;
    }

    camera_tr.rotation = Quat::from_axis_angle(Vec3::X, delta_state.pitch);
    pivot_tr.rotate_local_y(mouse_delta_yaw);

    let _up = Vec3::Y;
    let _right = pivot_tr.right().normalize();
    let _forward = _up.cross(_right).normalize();

    pivot_tr.translation.y = pivot_comp.camera_height + height(&pivot_tr.translation);
    let mut _pos_xz = Vec3::new(pivot_tr.translation.x, 0.0, pivot_tr.translation.z);
    let max_camera_xz = PLANET_MAX_PLAY_RADIUS;
    if _pos_xz.length() > max_camera_xz {
        _pos_xz = _pos_xz.normalize() * max_camera_xz;
        pivot_tr.translation.x = _pos_xz.x;
        pivot_tr.translation.z = _pos_xz.z;
    }
    let _target = pivot_tr.translation + _forward;
    pivot_tr.look_at(_target, _up);
}
