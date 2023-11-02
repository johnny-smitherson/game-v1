use bevy::prelude::*;

use bevy_egui::{egui, EguiContexts, EguiPlugin};

use super::terrain::TerrainSettings;
use bevy_inspector_egui::prelude::InspectorOptions;

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource)]
pub struct UiMenuState {
    #[reflect(ignore)]
    pub settings: TerrainSettings,
    pub enable_animation: bool,
    pub triangle_count: f32,
    pub mesh_count: f32,
    pub tri_compute_ms: f32,
    pub ui_prevent_input: bool,
    pub is_mouse_captured: bool,
}

pub fn egui_ui_system(mut egui_context: EguiContexts, mut ui_state: ResMut<UiMenuState>) {
    egui::Window::new("Piramidă").show(egui_context.ctx_mut(), |ui| {
        ui.label(
            "  Triangles: ".to_string()
                + ui_state.triangle_count.to_string().as_str()
                + "  Meshes: "
                + ui_state.mesh_count.to_string().as_str()
                + " COMPUTE MS: "
                + ui_state.tri_compute_ms.to_string().as_str(),
        );
        ui.label(" PREVENT INPUT: ".to_string() + ui_state.ui_prevent_input.to_string().as_str());
        ui.label("Planet Settings");

        ui.add(
            egui::Slider::new(&mut ui_state.settings.MAX_SPLIT_LEVEL, 3..=20)
                .text("MAX_SPLIT_LEVEL"),
        );

        ui.add(
            egui::Slider::new(&mut ui_state.settings.MIN_SPLIT_LEVEL, 0..=10)
                .text("MIN_SPLIT_LEVEL"),
        );

        ui.add(
            egui::Slider::new(&mut ui_state.settings.TESSELATION_VALUE, 1.0..=10.0)
                .text("TESSELATION_VALUE"),
        );

        ui.add(
            egui::Slider::new(&mut ui_state.settings.MIN_CAMERA_HEIGHT, 0.3..=3.0)
                .text("MIN_CAMERA_HEIGHT"),
        );
        ui.add(
            egui::Slider::new(&mut ui_state.settings.MAX_CAMERA_HEIGHT, 100.0..=500.0)
                .text("MAX_CAMERA_HEIGHT"),
        );
        ui.add(
            egui::Slider::new(&mut ui_state.settings.SPLIT_LAZY_COEF, 0.0..=0.45)
                .text("SPLIT_LAZY_COEF"),
        );
        ui.add(
            egui::Slider::new(&mut ui_state.settings.MIN_TRIANGLE_EDGE_SIZE, 0.01..=10.0)
                .text("MIN_TRIANGLE_EDGE_SIZE"),
        );
        ui.checkbox(&mut ui_state.enable_animation, "ENABLE ADNIMATION");
    });
    ui_state.ui_prevent_input = egui_context.ctx_mut().is_pointer_over_area();
}

pub fn allow_player_input(ui_state: Res<UiMenuState>) -> bool {
    !ui_state.ui_prevent_input
}

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .init_resource::<UiMenuState>()
            .register_type::<UiMenuState>()
            .add_systems(
                Update,
                (egui_ui_system, cursor_grab_click.run_if(allow_player_input)).chain(),
            );
    }
}

use bevy::window::{CursorGrabMode, PrimaryWindow};

fn cursor_grab_click(
    mouse: Res<Input<MouseButton>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = window_query.get_single_mut() {
        if mouse.pressed(MouseButton::Middle) {
            window.cursor.grab_mode = CursorGrabMode::Confined;
            // window.cursor.visible = false;
        } else {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
    } else {
        warn!("Primary window not found for `cursor_grab_click`!");
    }
}
