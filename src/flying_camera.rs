use bevy::ecs::event::ManualEventReader;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy_atmosphere::prelude::*;
use bevy_inspector_egui::prelude::InspectorOptions;
use core::f32::consts::PI;
use smart_default::SmartDefault;

use bevy::core_pipeline::bloom::BloomSettings;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::window::{CursorGrabMode, PrimaryWindow};

use bevy::{
    core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
    pbr::ScreenSpaceAmbientOcclusionBundle,
};

use crate::planet::TerrainSplitProbe;

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
            .add_systems(Update, (cursor_grab, daylight_cycle));
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
        let time = time.elapsed_seconds_wrapped() / 200.0;
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
            BloomSettings {
                ..default() // intensity: 0.02,
                            // scale: 0.5,
                            // knee: -3.23,
                            // threshold: 0.7,
            },
            // PickingCameraBundle::default();
            Name::new("THE CAMERA"),
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
