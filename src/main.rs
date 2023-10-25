mod game_assets;
mod height;
mod menu;
mod piramida;
mod planet;
mod player;
mod triangle;

use game_assets::GameAssetsPlugin;
use menu::MenuPlugin;
use planet::PlanetPlugin;
use player::PlayerPlugin;

use bevy::{
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use bevy_atmosphere::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

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
        // .add_systems(PreStartup, setup_world_scene)
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
        .add_plugins(GameAssetsPlugin)
        // ============
        // DIAGNOSTIC DEBUG LOGGING
        // =============
        // .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(ScreenDiagnosticsPlugin::default())
        .add_plugins(ScreenFrameDiagnosticsPlugin)
        .add_plugins(WorldInspectorPlugin::default())
        // .add_plugins(DefaultPickingPlugins)
        // .add_plugin(DebugCursorPickingPlugin) // <- Adds the debug cursor (optional)
        // .add_plugin(DebugEventsPickingPlugin) // <- Adds debug event logging (optional)
        .run();

    warn!("game exiting");
}

fn setup_world_scene(mut commands: Commands) {
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
