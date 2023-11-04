use bevy::prelude::*;
use bevy_inspector_egui::InspectorOptions;

use crate::{
    camera_extra::{spawn_extra_camera, ExtraCamera},
    menu::UiMarkHoverBundle,
};

pub struct MinimapPlugin;
impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_minimap_camera)
            .add_systems(PostStartup, setup_minimap_ui.after(setup_minimap_camera));
    }
}

#[derive(Component, Reflect, InspectorOptions)]
pub struct MinimapCamera;

const MINIMAP_REZ: u32 = 256;

fn setup_minimap_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    spawn_extra_camera(
        &mut commands,
        &mut images,
        MINIMAP_REZ,
        (MinimapCamera, Name::new("Minimap Camera")),
    );
}

fn setup_minimap_ui(mut commands: Commands, camera_q: Query<&ExtraCamera, With<MinimapCamera>>) {
    let camera_comp = camera_q.get_single().expect("wtf no minimap camera");

    let camera_bundle = camera_comp.render_target_image_bundle();
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::VMin(35.0),
                    height: Val::VMin(35.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    position_type: PositionType::Absolute,
                    column_gap: Val::Px(5.0),
                    right: Val::VMin(1.0),
                    top: Val::VMin(1.0),
                    ..default()
                },
                ..default()
            },
            UiMarkHoverBundle::default(),
        ))
        .with_children(|parent| {
            parent.spawn(camera_bundle);
        });
}
