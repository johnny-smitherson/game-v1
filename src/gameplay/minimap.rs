use bevy::{prelude::*};
use bevy_inspector_egui::InspectorOptions;

use crate::{
    camera_extra::{spawn_extra_camera, ExtraCamera},
    camera_flying::FlyingCamera,
    menu::UiMarkHoverBundle,
    terrain::MOUNTAIN_HEIGHT,
};

use super::tank::{PlayerControlledTank, Tank};

pub struct MinimapPlugin;
impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_minimap_camera)
            .add_systems(PostStartup, setup_minimap_ui.after(setup_minimap_camera))
            .add_systems(Update, update_minimap_position);
    }
}

#[derive(Component, Reflect, InspectorOptions)]
pub struct MinimapCamera;

const MINIMAP_REZ: u32 = 512;

fn setup_minimap_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut gizmo_config: ResMut<GizmoConfig>,
) {
    spawn_extra_camera(
        &mut commands,
        &mut images,
        MINIMAP_REZ,
        (MinimapCamera, Name::new("Minimap Camera")),
    );

    gizmo_config.line_width = 5.0;
    gizmo_config.enabled = true;
    // gizmo_config.render_layers = RenderLayers::layer(1);
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

fn update_minimap_position(
    mut minimap: Query<&mut Transform, With<MinimapCamera>>,
    tanks: Query<(&Transform, Option<&PlayerControlledTank>), (With<Tank>, Without<MinimapCamera>)>,

    flying_camera_q: Query<&GlobalTransform, With<FlyingCamera>>,
    mut gizmos: Gizmos,
) {
    let Ok(mut minimap_pos) = minimap.get_single_mut() else {
        return;
    };

    let mut item_pos: Vec<_> = tanks.iter().map(|(t, _)| t.translation).collect();
    for tr in flying_camera_q.iter() {
        item_pos.push(tr.compute_transform().translation);
    }
    if item_pos.is_empty() {
        return;
    }
    let center_position = item_pos.iter().sum::<Vec3>() / item_pos.len() as f32;
    let (min_pos, max_pos) = bbox_from_points(&item_pos);
    let spread = (max_pos - min_pos + Vec3::ONE * 100.0).length() / 2.0 + MOUNTAIN_HEIGHT;
    const SPREAD_MULTIPLIER: f32 = 1.5;

    let minimap_y = center_position.y + spread * SPREAD_MULTIPLIER + MOUNTAIN_HEIGHT;
    minimap_pos.translation = Vec3::new(center_position.x, minimap_y, center_position.z);

    // ** GIZMOS - camera arrow
    for camera in flying_camera_q.iter() {
        let line_long: f32 = spread * 0.25;
        let line_short: f32 = line_long * 0.25;
        let tr = camera.compute_transform();
        let arrow_tip = tr.translation + tr.back() * 0.01 + Vec3::Y * MOUNTAIN_HEIGHT;
        let arrow_base = arrow_tip + tr.back() * line_long;
        let arrow_left = arrow_tip + (tr.left() + tr.back()) * line_short;
        let arrow_right = arrow_tip + (tr.right() + tr.back()) * line_short;
        let color = Color::WHITE;
        gizmos.line(arrow_tip, arrow_base, color);
        gizmos.line(arrow_tip, arrow_left, color);
        gizmos.line(arrow_tip, arrow_right, color);
    }

    // ** GIZMOS - tank
    for (tank_tr, is_player) in tanks.iter() {
        let color = if is_player.is_some() {
            Color::GREEN
        } else {
            Color::RED
        };

        let circle_radius = spread * 0.05;
        let circle_count = 3;
        for i in 1..=circle_count {
            let scale = i as f32 / circle_count as f32;
            let circle_center =
                tank_tr.translation + Vec3::Y * (circle_radius + MOUNTAIN_HEIGHT) * scale;
            gizmos.circle(circle_center, Vec3::Y, circle_radius * scale, color);
        }
    }
}

fn bbox_from_points(points: &Vec<Vec3>) -> (Vec3, Vec3) {
    let mut min = Vec3::INFINITY;
    let mut max = Vec3::NEG_INFINITY;

    for point in points {
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        min.z = min.z.min(point.z);

        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
        max.z = max.z.max(point.z);
    }

    (min, max)
}
