mod assets;
mod audio;
mod camera_extra;
mod camera_flying;
mod gameplay;
mod menu;
mod oct_tree;
mod piramida;
mod planet;
mod raycast;
mod terrain;
mod triangle;

use std::time::Duration;

use assets::GameAssetsPlugin;
use camera_flying::FlyingCameraPlugin;
use gameplay::GameplayPlugin;
use menu::MenuPlugin;
use planet::PlanetPlugin;

use bevy::{
    log::LogPlugin,
    prelude::*,
    render::{
        settings::{WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
    window::{PresentMode, WindowResolution},
    winit::WinitSettings,
};

use crate::audio::GameAudioPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

pub fn create_game_app(disable_graphics: bool) -> App {
    let mut wgpu_settings = WgpuSettings::default();
    wgpu_settings
        .features
        .set(WgpuFeatures::VERTEX_WRITABLE_STORAGE, true);

    let mut window_settings = Some(Window {
        present_mode: PresentMode::AutoVsync,
        window_level: bevy::window::WindowLevel::AlwaysOnBottom,
        resolution: WindowResolution::new(1920., 1080.),
        title: "Game V3".into(),
        // Tells wasm to resize the window according to the available canvas
        fit_canvas_to_parent: true,
        // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
        prevent_default_event_handling: false,
        focused: false,
        position: WindowPosition::Centered(MonitorSelection::Index(1)),
        ..default()
    });

    if disable_graphics {
        // wgpu_settings.backends = None;
        // window_settings = None;
    }

    let mut default_plugins = DefaultPlugins
        .set(RenderPlugin { wgpu_settings })
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
            primary_window: window_settings,
            ..default()
        })
        .build();

    let mut app = App::new();
    // ==============
    // DEFAULT PLUGINS + WINDOW SETTINGS
    // ==============
    app.add_plugins(default_plugins)
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
        // .add_plugins(RapierDebugRenderPlugin::default())
        // ==============
        // GAME PLUGINS
        // ==============
        .add_plugins(MenuPlugin)
        .add_plugins(FlyingCameraPlugin)
        .add_plugins(camera_extra::ExtraCameraPlugin)
        .add_plugins(GameAudioPlugin)
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
    ;
    app
}
