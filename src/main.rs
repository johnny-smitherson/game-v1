mod height;
mod menu;
mod piramida;
mod planet;
mod player;
mod triangle;

use menu::MenuPlugin;
use planet::PlanetPlugin;
use player::PlayerPlugin;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        // ==============
        // DEFAULT PLUGINS + WINDOW SETTINGS
        // ==============
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::AutoVsync,
                        resolution: WindowResolution::new(1920., 1080.),
                        title: "The Wuindow!".into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(Msaa::Sample4)
        .add_systems(PreStartup, setup_world_scene)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.05,
        })
        // ==============
        // PHYSICS AND SHIT
        // ==============
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        // ==============
        // GAME PLUGINS
        // ==============
        .add_plugins(MenuPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(PlanetPlugin)
        // ============
        // DIAGNOSTIC DEBUG LOGGING
        // =============
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(WorldInspectorPlugin::default())
        // .add_plugins(DefaultPickingPlugins)
        // .add_plugin(DebugCursorPickingPlugin) // <- Adds the debug cursor (optional)
        // .add_plugin(DebugEventsPickingPlugin) // <- Adds debug event logging (optional)
        .run();

    warn!("game exiting");
}

fn setup_world_scene(mut commands: Commands) {
    // // 3 suns because night sucks
    // commands.spawn(DirectionalLightBundle {
    //     directional_light: DirectionalLight {
    //         color: Color::rgb(0.99, 0.9, 0.9),
    //         illuminance: 6789.0,
    //         shadows_enabled: true,
    //         ..Default::default()
    //     },
    //     transform: Transform::from_rotation(
    //         Quat::from_rotation_x(1.) * Quat::from_rotation_y(0.) * Quat::from_rotation_z(1.),
    //     ),
    //     ..default()
    // });

    // commands.spawn(DirectionalLightBundle {
    //     directional_light: DirectionalLight {
    //         color: Color::rgb(0.9, 0.99, 0.9),
    //         illuminance: 7890.0,
    //         shadows_enabled: true,
    //         ..Default::default()
    //     },
    //     transform: Transform::from_rotation(
    //         Quat::from_rotation_x(-1.) * Quat::from_rotation_y(1.) * Quat::from_rotation_z(-1.),
    //     ),
    //     ..default()
    // });

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::rgb(0.95, 0.9, 0.99),
                illuminance: 8906.0,
                shadows_enabled: true,
                ..Default::default()
            },
            transform: Transform::from_rotation(
                Quat::from_rotation_x(1.) * Quat::from_rotation_y(3.) * Quat::from_rotation_z(0.5),
            ),
            ..default()
        },
        Name::new("THE SUN"),
    ));
}
