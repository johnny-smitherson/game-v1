use bevy::prelude::*;

use crate::flying_camera::{FlyingCameraInputState, FlyingCameraPivot};

use super::tank::PlayerControlledTank;

pub struct KeyboardShortcutsPlugin;
impl Plugin for KeyboardShortcutsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (center_camera_on_player_tank,));
    }
}

fn center_camera_on_player_tank(
    mut camera_pivot: Query<(&mut Transform, &mut FlyingCameraPivot), With<FlyingCameraPivot>>,
    player_tank: Query<&Transform, (With<PlayerControlledTank>, Without<FlyingCameraPivot>)>,
    mut camera_state: ResMut<FlyingCameraInputState>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::F1) {
        if let Ok((mut camera_transform, mut camera_pivot)) = camera_pivot.get_single_mut() {
            if let Ok(player_tank) = player_tank.get_single() {
                camera_transform.translation = player_tank.translation + Vec3::new(25.0, 0.0, 25.0);
                camera_transform.look_at(player_tank.translation, Vec3::Y);
                camera_state.pitch = -0.4;
                camera_pivot.camera_height = 25.0;
            }
        }
    }
}
