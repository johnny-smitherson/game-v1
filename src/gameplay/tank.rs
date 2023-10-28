use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    game_assets::GameSceneAssets,
    planet::TerrainSplitProbe,
    terrain::{apply_height, height},
};
use core::f32::consts::PI;

use smart_default::SmartDefault;

pub struct TankPlugin;
impl Plugin for TankPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TankGravity>()
            .add_systems(Startup, tank_setup)
            .add_systems(
                Update,
                (control_player_tank_mvmt, tank_gravity_update).chain(),
            )
            .add_systems(PostUpdate, read_tank_gravity_result)
            .add_systems(PreUpdate, tank_fix_above_terrain);
    }
}

#[derive(Reflect, Component, Default)]
pub struct PlayerControlledTank;

fn control_player_tank_mvmt(
    mut tank: Query<
        (&mut KinematicCharacterController, &mut Transform, &mut Tank),
        With<PlayerControlledTank>,
    >,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut gizmos: Gizmos,
    mut gizmo_config: ResMut<GizmoConfig>,
) {
    let mut _delta_bearing: f32 = 0.0;
    let mut _delta_adv: f32 = 0.0;
    let mut _delta_elev: f32 = 0.0;

    const elevation_speed: f32 = 0.5;
    const bearing_speed: f32 = 0.9;
    const tank_mvmt_speed: f32 = 8.5;

    for key in keys.get_pressed() {
        match key {
            KeyCode::ShiftRight => {
                _delta_elev += elevation_speed * time.delta_seconds();
            }
            KeyCode::ControlRight => {
                _delta_elev -= elevation_speed * time.delta_seconds();
            }
            KeyCode::Up => {
                _delta_adv += tank_mvmt_speed * time.delta_seconds();
            }
            KeyCode::Down => {
                _delta_adv -= tank_mvmt_speed * time.delta_seconds();
            }
            KeyCode::Left => {
                _delta_bearing += bearing_speed * time.delta_seconds();
            }
            KeyCode::Right => {
                _delta_bearing -= bearing_speed * time.delta_seconds();
            }
            _ => (),
        }
    }
    let (mut tank_controller, mut tank_transform, mut tank_data) = tank.single_mut();
    tank_data.bearing += _delta_bearing;
    tank_transform.rotation = Quat::from_rotation_y(tank_data.bearing);

    tank_data.elevation += _delta_elev;
    tank_data.elevation = tank_data.elevation.clamp(0.0, PI / 2.0);
    let elevation = tank_data.elevation;

    tank_controller.translation = Some(-tank_transform.right() * _delta_adv);

    const GIZMO_FIRE_LEN: f32 = 10.0;
    const GIZMO_EMPTY_RADIUS: f32 = 5.0;
    tank_data.fire_direction =
        ((-tank_transform.right()) * elevation.cos() + Vec3::Y * elevation.sin()).normalize();
    let gizmo_origin = tank_transform.translation;
    let gizmo_fire_src = gizmo_origin + tank_data.fire_direction * GIZMO_EMPTY_RADIUS;
    let gizmo_fire_end =
        gizmo_origin + tank_data.fire_direction * (GIZMO_FIRE_LEN + GIZMO_EMPTY_RADIUS);
    let gizmo_blue_proj_src = Vec3::new(gizmo_fire_src.x, gizmo_origin.y, gizmo_fire_src.z);
    let gizmo_blue_proj_end = Vec3::new(gizmo_fire_end.x, gizmo_origin.y, gizmo_fire_end.z);
    tank_data.fire_origin = gizmo_fire_src;

    gizmos.line(gizmo_fire_src, gizmo_fire_end, Color::RED);
    gizmos.line(gizmo_blue_proj_src, gizmo_blue_proj_end, Color::BLUE);
    gizmo_config.line_width = 3.0;
}

#[derive(Reflect, Component, SmartDefault)]
pub struct Tank {
    #[default(PI/4.0)]
    elevation: f32,
    #[default(0.0)]
    bearing: f32,

    pub fire_direction: Vec3,
    pub fire_origin: Vec3,
}

#[derive(Reflect, Component, Default)]
pub struct TankGravity {
    is_grounded: bool,
    fall_time: f32,
}

fn tank_setup(mut commands: Commands, scene_assets: Res<GameSceneAssets>) {
    let collider_size: f32 = 0.5;
    let tank_collider = Collider::cuboid(collider_size, collider_size / 5.0, collider_size);

    let tank_model = SceneBundle {
        scene: scene_assets
            .scenes
            .get("ORIGINAL/Tanks and Armored Vehicle.glb")
            .expect("KEY NOT FOUND")
            .clone(),
        ..Default::default()
    };

    let tank_controller = KinematicCharacterController {
        // The character offset is set to 0.01.
        offset: CharacterLength::Absolute(0.01),
        // Don’t allow climbing slopes larger than 45 degrees.
        max_slope_climb_angle: 60.0_f32.to_radians(),
        // Automatically slide down on slopes smaller than 30 degrees.
        // min_slope_slide_angle: 0.0_f32.to_radians(),
        // snap to ground 0.5
        snap_to_ground: Some(CharacterLength::Relative(5.5)),
        ..default()
    };

    let tank_spawn_pos = apply_height(&Vec3::ZERO) + Vec3::Y * (collider_size + 10.0);

    let tank_id = commands
        .spawn((Tank::default(), SpatialBundle::default()))
        .insert(Transform::from_translation(tank_spawn_pos))
        .insert(TankGravity::default())
        .insert((
            RigidBody::KinematicPositionBased,
            tank_controller,
            tank_collider, // tank_model,
        ))
        .insert(PlayerControlledTank)
        .insert(Name::new("Player Tank"))
        .insert(TerrainSplitProbe)
        .id();
    commands
        .spawn((tank_model, Name::new("Tank Model")))
        .insert(Transform::from_translation(Vec3::Y * -0.5_f32).with_scale(Vec3::ONE * 0.5_f32))
        .set_parent(tank_id); //.insert(Transform::from_scale(Vec3::ONE * 0.25));
}

const GRAVITY: f32 = 9.8_f32;

fn tank_gravity_update(
    mut tanks: Query<(&mut KinematicCharacterController, &mut TankGravity), With<TankGravity>>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (mut tank_transform, mut gravity) in tanks.iter_mut() {
        if !gravity.is_grounded {
            gravity.fall_time += dt;
            tank_transform.translation = Some(-Vec3::Y * gravity.fall_time * GRAVITY);
        }
    }
}

fn read_tank_gravity_result(
    mut controllers: Query<(&mut TankGravity, &KinematicCharacterControllerOutput)>,
) {
    for (mut grav, output) in controllers.iter_mut() {
        grav.is_grounded = output.grounded;
        if grav.is_grounded {
            grav.fall_time = 0.;
        }
    }
}

fn tank_fix_above_terrain(mut transforms: Query<&mut Transform, With<TankGravity>>) {
    for mut transform in transforms.iter_mut() {
        const reset_below: f32 = 2.0;
        let terrain_height = height(&Vec3::ZERO);
        if transform.translation.y < terrain_height - reset_below {
            transform.translation.y = terrain_height + reset_below * 3.;
        }
    }
}
