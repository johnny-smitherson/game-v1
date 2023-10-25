use bevy::ecs::event::ManualEventReader;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy_atmosphere::prelude::*;
use bevy_inspector_egui::prelude::InspectorOptions;
use core::f32::consts::PI;

/// A marker component used in queries when you want flycams and not other cameras
#[derive(Component)]
pub struct FlyCam;

// Marker for updating the position of the light, not needed unless we have multiple lights
#[derive(Component)]
struct Sun;

// Timer for updating the daylight cycle (updating the atmosphere every frame is slow, so it's better to do incremental changes)
#[derive(Resource)]
struct CycleTimer(Timer);

fn daylight_cycle(
    mut atmosphere: AtmosphereMut<Nishita>,
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut timer: ResMut<CycleTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    if timer.0.finished() {
        let time = time.elapsed_seconds_wrapped() / 200.0;
        let t = PI / 2.0 + time.sin() * 0.35;

        atmosphere.sun_position = Vec3::new(0., t.sin(), t.cos());

        for (mut light_trans, mut directional) in query.iter_mut() {
            light_trans.rotation = Quat::from_rotation_x(-t);
            directional.illuminance = t.sin().max(0.0).powf(2.0) * 8000.0;
        }
    }
}

fn setup_sun(mut commands: Commands) {
    // Our Sun
    commands.spawn((
        DirectionalLightBundle {
            transform: Transform::from_rotation(Quat::from_rotation_x(-PI / 2.0)),
            directional_light: DirectionalLight {
                illuminance: 8000.0,
                ..default()
            },
            ..Default::default()
        },
        Sun,
    ));
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .register_type::<InputState>()
            .init_resource::<MovementSettings>()
            .register_type::<MovementSettings>()
            .add_plugins(AtmospherePlugin)
            .insert_resource(AtmosphereModel::default())
            .insert_resource(CycleTimer(Timer::new(
                bevy::utils::Duration::from_millis(500),
                TimerMode::Repeating,
            )))
            .add_systems(PreStartup, (setup_player, setup_sun))
            .add_systems(Update, (cursor_grab, daylight_cycle));
    }
}

// use bevy_flycam::FlyCam;
// use bevy_flycam::NoGrabNoPlayerPlugin;

/// Marker Component for the Entity that is our Player
#[derive(Bundle, Default)]
pub struct PlayerBundle {
    pub spatial: SpatialBundle,
    pub player_comp: PlayerComponent,
}

#[derive(Component)]
pub struct PlayerComponent {
    pub camera_height: f32,
}

impl Default for PlayerComponent {
    fn default() -> Self {
        Self {
            camera_height: 10.0,
        }
    }
}

use bevy::core_pipeline::bloom::BloomSettings;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::window::{CursorGrabMode, PrimaryWindow};

fn setup_player(mut commands: Commands) {
    warn!("PLAYER SETUP SYSTEM...");

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
            FlyCam,
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
        .id();

    let player = commands
        .spawn((PlayerBundle { ..default() }, Name::new("THE PLAYER")))
        .id();
    commands.entity(camera).set_parent(player);
    info!("camera: {:?} player: {:?}", camera, player);
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
pub struct InputState {
    pub pitch: f32,
    pub yaw: f32,
    #[reflect(ignore)]
    pub reader_motion: ManualEventReader<MouseMotion>,
}

/// Mouse sensitivity and movement speed

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource)]

pub struct MovementSettings {
    pub sensitivity: f32,
    pub speed: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 4.02,
        }
    }
}
