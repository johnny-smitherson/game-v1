use bevy::ecs::event::ManualEventReader;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy_inspector_egui::prelude::InspectorOptions;
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
use bevy_rapier3d::prelude::*;

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

// use bevy_atmosphere::prelude::AtmosphereCamera;
/// Spawns the `Camera3dBundle` to be controlled
fn setup_player(mut commands: Commands) {
    warn!("PLAYER SETUP SYSTEM...");

    let camera = commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    hdr: true,
                    ..default()
                },

                tonemapping: Tonemapping::BlenderFilmic,
                ..Default::default()
            },
            FlyCam,
            // AtmosphereCamera::default(),
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

/// A marker component used in queries when you want flycams and not other cameras
#[derive(Component)]
pub struct FlyCam;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup_player)
            .init_resource::<InputState>()
            .register_type::<InputState>()
            .init_resource::<MovementSettings>()
            .register_type::<MovementSettings>()
            .add_systems(Update, cursor_grab);
    }
}
