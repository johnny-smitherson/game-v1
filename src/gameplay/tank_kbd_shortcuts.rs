use bevy::prelude::*;

use crate::{
    flying_camera::{FlyingCameraInputState, FlyingCameraPivot},
    menu::mouse_not_over_menu,
    raycast::TerrainRaycastResult,
};

use super::{
    events::{TankCommandEvent, TankCommandEventType},
    tank::{PlayerControlledTank, Tank},
};

pub struct KeyboardShortcutsPlugin;
impl Plugin for KeyboardShortcutsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, (read_keys_for_player_tank_control,))
            .add_systems(
                Update,
                (
                    center_camera_on_player_tank,
                    aim_tank_on_click.run_if(mouse_not_over_menu),
                ),
            );
    }
}

fn center_camera_on_player_tank(
    mut camera_pivot: Query<(&mut Transform, &mut FlyingCameraPivot), With<FlyingCameraPivot>>,
    player_tank: Query<
        (&Transform, &Tank),
        (With<PlayerControlledTank>, Without<FlyingCameraPivot>),
    >,
    mut camera_state: ResMut<FlyingCameraInputState>,
    keys: Res<Input<KeyCode>>,
) {
    let camera_height = 35.0_f32;

    if keys.just_pressed(KeyCode::F1) {
        if let Ok((mut camera_transform, mut camera_pivot)) = camera_pivot.get_single_mut() {
            if let Ok((player_tank, _tank_data)) = player_tank.get_single() {
                camera_transform.translation =
                    player_tank.translation + Vec3::new(camera_height, 0.0, camera_height);
                camera_transform.look_at(player_tank.translation, Vec3::Y);
                camera_state.pitch = -0.3;
                camera_pivot.camera_height = camera_height;
            }
        }
    }
}

fn read_keys_for_player_tank_control(
    keys: Res<Input<KeyCode>>,
    mut tank_command_events: EventWriter<TankCommandEvent>,
    tank_query: Query<Entity, With<PlayerControlledTank>>,
) {
    if let Ok(tank_entity) = tank_query.get_single() {
        if keys.just_pressed(KeyCode::Space) {
            tank_command_events.send(TankCommandEvent {
                tank_entity,
                event_type: TankCommandEventType::Fire,
            });
        }
        for key in keys.get_pressed() {
            let event_type = match key {
                KeyCode::Up => TankCommandEventType::ElevationPlus,
                KeyCode::Down => TankCommandEventType::ElevationMinus,
                KeyCode::Right => TankCommandEventType::BearingRight,
                KeyCode::Left => TankCommandEventType::BearingLeft,
                KeyCode::Numpad8 => TankCommandEventType::MoveForward,
                KeyCode::Numpad2 => TankCommandEventType::MoveBack,
                KeyCode::Numpad4 => TankCommandEventType::MoveLeft,
                KeyCode::Numpad6 => TankCommandEventType::MoveRight,
                KeyCode::ShiftRight | KeyCode::Plus | KeyCode::Equals => {
                    TankCommandEventType::PowerPlus
                }
                KeyCode::ControlRight | KeyCode::Minus => TankCommandEventType::PowerMinus,
                _ => continue,
            };
            tank_command_events.send(TankCommandEvent {
                tank_entity,
                event_type,
            });
        }
    }
}

fn aim_tank_on_click(
    mouse: Res<Input<MouseButton>>,
    mut tank_command_events: EventWriter<TankCommandEvent>,
    tank_query: Query<Entity, With<PlayerControlledTank>>,
    terrain_raycast: Res<TerrainRaycastResult>,
) {
    if mouse.pressed(MouseButton::Left) {
        if let Ok(tank_entity) = tank_query.get_single() {
            if let Some(intersection) = &terrain_raycast.intersection {
                let pos = intersection.position();
                tank_command_events.send(TankCommandEvent {
                    event_type: TankCommandEventType::AimAtPoint(pos),
                    tank_entity,
                })
            }
        }
    }
}
