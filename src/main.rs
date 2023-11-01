mod flying_camera;
mod game_assets;
mod gameplay;
mod menu;
mod piramida;
mod planet;
mod raycast;
mod terrain;
mod triangle;

use std::time::Duration;

use flying_camera::FlyingCameraPlugin;
use game_assets::GameAssetsPlugin;
use gameplay::GameplayPlugin;
use menu::MenuPlugin;
use planet::PlanetPlugin;

use bevy::{
    prelude::*,
    render::{
        settings::{WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
    window::{PresentMode, WindowResolution},
    winit::WinitSettings,
};

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_raycast::low_latency_window_plugin;
use bevy_rapier3d::prelude::*;
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

fn main() {
    let mut wgpu_settings = WgpuSettings::default();
    wgpu_settings
        .features
        .set(WgpuFeatures::VERTEX_WRITABLE_STORAGE, true);

    App::new()
        // ==============
        // DEFAULT PLUGINS + WINDOW SETTINGS
        // ==============
        .add_plugins(
            DefaultPlugins
                .set(low_latency_window_plugin())
                .set(RenderPlugin { wgpu_settings })
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::AutoVsync,
                        window_level: bevy::window::WindowLevel::AlwaysOnBottom,
                        resolution: WindowResolution::new(1920., 1080.),
                        title: "The Wuindow!".into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(WinitSettings {
            focused_mode: bevy::winit::UpdateMode::Continuous,
            unfocused_mode: bevy::winit::UpdateMode::ReactiveLowPower {
                max_wait: Duration::from_millis(1000),
            },
            ..default()
        })
        // .insert_resource(Msaa::Sample4)
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
        .add_plugins(FlyingCameraPlugin)
        .add_plugins(PlanetPlugin)
        .add_plugins(GameAssetsPlugin)
        .add_plugins(GameplayPlugin)
        .add_plugins(raycast::RaycastPlugin)
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
