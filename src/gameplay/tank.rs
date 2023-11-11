use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    assets::GameSceneAssets,
    gameplay::bullet_physics::{GRAVITY_MAGNITUDE, TANK_DENSITY},
    menu::mouse_not_over_menu,
    planet::TerrainSplitProbe,
    terrain::{apply_height, height},
};
use core::f32::consts::PI;
use std::time::Duration;

use smart_default::SmartDefault;

use super::{
    bullet_physics::{
        compute_ballistic_solution, BulletSolutions, GRAVITY_SCALE, TANK_BULLET_SPEED_PER_POWER,
    },
    events::{BulletHitEvent, TankCommandEvent, TankCommandEventType},
};

use bevy_spatial::{kdtree::KDTree3, AutomaticUpdate, SpatialAccess, TransformMode};

pub struct TankPlugin;
impl Plugin for TankPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TankGravity>()
            .register_type::<Tank>()
            .register_type::<PlayerControlledTank>()
            .add_systems(Startup, tank_setup)
            .add_systems(PreUpdate, (tank_fix_above_terrain, on_tank_hit))
            .add_systems(
                Update,
                (
                    control_tank_aim.run_if(mouse_not_over_menu),
                    debug_show_tank_aim,
                    (control_tank_mvmt, tank_gravity_update).chain(),
                ),
            )
            .add_systems(PostUpdate, read_tank_gravity_result)
            // Spatial tracking of tanks for damage
            .add_plugins(
                AutomaticUpdate::<Tank>::new()
                    .with_frequency(Duration::from_secs_f32(1.0 / 60.0))
                    .with_transform(TransformMode::GlobalTransform),
            );
    }
}

#[derive(Reflect, Component, Default)]
pub struct PlayerControlledTank;

#[derive(Reflect, Component, SmartDefault)]
pub struct Tank {
    #[default(PI/4.0)]
    pub elevation: f32,
    #[default(0.0)]
    pub bearing: f32,
    #[default(1000.0)]
    pub power: f32,

    pub fire_direction: Vec3,
    pub fire_origin: Vec3,

    pub fire_solutions: Option<BulletSolutions>,
}

#[derive(Reflect, Component, Default)]
pub struct TankGravity {
    is_grounded: bool,
    fall_time: f32,
}

const BULLET_DAMAGE_DISTANCE: f32 = 26.0;

fn on_tank_hit(tank_tree: Res<KDTree3<Tank>>, mut events: EventReader<BulletHitEvent>) {
    for event in events.iter() {
        for (tank_pos, tank_ent) in
            tank_tree.within_distance(event.bullet_pos, BULLET_DAMAGE_DISTANCE)
        {
            let bullet_dist = (event.bullet_pos - tank_pos).length();
            if let Some(tank_ent) = tank_ent {
                info!(
                    "hit {:?} at {:?}, dist {:?}",
                    tank_ent, tank_pos, bullet_dist
                );
            } else {
                warn!("WTF got hit but no entity, why?");
            }
            // pos: Vec3
            // do something with the nearest entity here
        }
    }
}

fn control_tank_aim(
    mut tank_q: Query<(&mut Tank, &Transform), With<Tank>>,
    mut tank_command_events: EventReader<TankCommandEvent>,
) {
    for event in tank_command_events.iter() {
        if let TankCommandEventType::AimAtPoint(aim_pos) = event.event_type {
            if let Ok((mut tank, _tank_tr)) = tank_q.get_mut(event.tank_entity) {
                let _tank_pos = tank.fire_origin; //  &tank_tr.translation;
                                                  // let _tank_pos = apply_height(&_tank_pos);
                let diff = aim_pos - _tank_pos;
                let bearing = diff.x.atan2(diff.z);
                // compute elevation ignoring Y diff
                // https://qph.cf2.quoracdn.net/main-qimg-9aa63a48016d31489787c9c36f138c79
                let range = Vec2::new(diff.x, diff.z).length();
                let speed = tank.power * TANK_BULLET_SPEED_PER_POWER;
                tank.bearing = bearing;

                let solutions = compute_ballistic_solution(range, diff.y, speed);
                tank.fire_solutions = Some(solutions.clone());
                if let Some(s) = solutions.low_sol {
                    tank.elevation = s.elevation;
                } else if let Some(s) = solutions.high_sol {
                    tank.elevation = s.elevation;
                } else if let Some(s) = solutions.err_sol {
                    tank.elevation = s.elevation;
                } else {
                    panic!("solution generator did not return err_sol");
                }
            }
        }
    }
}
fn debug_line_strip(gizmos: &mut Gizmos, trajectory: &[Vec3], color: &Color) {
    for i in 0..trajectory.len() - 1 {
        let point_a = trajectory[i];
        let point_b = trajectory[i + 1];
        gizmos.line(point_a, point_b, *color);
    }
}
fn debug_show_tank_aim(
    tanks: Query<(&Transform, &Tank), With<PlayerControlledTank>>,
    mut gizmos: Gizmos,
) {
    let mut draw_trajectory = |traj: &Vec<Vec2>, pos, bearing: f32, color| {
        let traj_3d: Vec<Vec3> = traj
            .iter()
            .map(|v2| Vec3::new(v2.x * bearing.sin(), v2.y, v2.x * bearing.cos()) + pos)
            .collect();
        // gizmos.linestrip(traj_3d, color);
        debug_line_strip(&mut gizmos, &traj_3d, &color);
    };
    for (_tank_tr, tank) in tanks.iter() {
        let tank_pos = tank.fire_origin; // tank_tr.translation;
        if let Some(solutions) = &tank.fire_solutions {
            if let Some(solution) = &solutions.low_sol {
                draw_trajectory(
                    &solution.trajectory,
                    tank_pos,
                    tank.bearing,
                    Color::YELLOW_GREEN,
                );
            }
            if let Some(solution) = &solutions.high_sol {
                draw_trajectory(
                    &solution.trajectory,
                    tank_pos,
                    tank.bearing,
                    Color::DARK_GREEN,
                );
            }
            if let Some(solution) = &solutions.err_sol {
                draw_trajectory(
                    &solution.trajectory,
                    tank_pos,
                    tank.bearing,
                    Color::ORANGE_RED,
                );
            }
        }
    }
}
fn control_tank_mvmt(
    mut tank: Query<
        (
            Entity,
            &mut KinematicCharacterController,
            &mut Transform,
            &mut Tank,
        ),
        With<Tank>,
    >,
    mut tank_command_events: EventReader<TankCommandEvent>,
    time: Res<Time>,
    mut gizmos: Gizmos,
    // mut gizmo_config: ResMut<GizmoConfig>,
) {
    // event reader remembers what it iterated through, so let's clone it
    let events: Vec<_> = tank_command_events.iter().collect();

    for (tank_entity, mut tank_controller, mut tank_transform, mut tank_data) in tank.iter_mut() {
        let mut _delta_bearing: f32 = 0.0;
        let mut _delta_adv: f32 = 0.0;
        let mut _delta_turn: f32 = 0.0;
        let mut _delta_elev: f32 = 0.0;
        let mut _delta_power: f32 = 0.0;

        const ELEVATION_SPEED: f32 = 0.7;
        const BEARING_SPEED: f32 = 1.3;
        const TANK_MVMT_SPEED: f32 = 8.5;
        const POWER_CHANGE_SPEED: f32 = 115.5;

        for event in events.iter() {
            if tank_entity != event.tank_entity {
                continue;
            }
            match event.event_type {
                TankCommandEventType::ElevationPlus => {
                    _delta_elev += ELEVATION_SPEED * time.delta_seconds();
                }
                TankCommandEventType::ElevationMinus => {
                    _delta_elev -= ELEVATION_SPEED * time.delta_seconds();
                }
                TankCommandEventType::MoveForward => {
                    _delta_adv += TANK_MVMT_SPEED * time.delta_seconds();
                }
                TankCommandEventType::MoveBack => {
                    _delta_adv -= TANK_MVMT_SPEED * time.delta_seconds();
                }
                TankCommandEventType::MoveLeft => {
                    _delta_turn -= BEARING_SPEED * time.delta_seconds();
                }
                TankCommandEventType::MoveRight => {
                    _delta_turn += BEARING_SPEED * time.delta_seconds();
                }
                TankCommandEventType::PowerPlus => {
                    _delta_power += POWER_CHANGE_SPEED * time.delta_seconds();
                }
                TankCommandEventType::PowerMinus => {
                    _delta_power -= POWER_CHANGE_SPEED * time.delta_seconds();
                }
                TankCommandEventType::BearingLeft => {
                    _delta_bearing += BEARING_SPEED * time.delta_seconds();
                }
                TankCommandEventType::BearingRight => {
                    _delta_bearing -= BEARING_SPEED * time.delta_seconds();
                }
                _ => (),
            }
        }

        // increment bearing
        tank_data.bearing += _delta_bearing;
        tank_data.bearing %= PI * 2.0;
        if tank_data.bearing < -PI {
            tank_data.bearing += PI * 2.0;
        } else if tank_data.bearing > PI {
            tank_data.bearing -= PI * 2.0;
        }
        // increment tank
        tank_transform.rotate(Quat::from_rotation_y(-_delta_turn));

        // elevation
        tank_data.elevation += _delta_elev;
        tank_data.elevation = tank_data.elevation.clamp(-PI / 4.0, PI / 2.0);

        tank_data.power += _delta_power;
        tank_data.power = tank_data.power.clamp(0.0, 1000.0);
        let elevation = tank_data.elevation;

        tank_controller.translation = Some(-tank_transform.right() * _delta_adv);

        const GIZMO_FIRE_LEN: f32 = 10.0;
        const GIZMO_EMPTY_RADIUS: f32 = 2.0;
        tank_data.fire_direction = Quat::from_rotation_y(tank_data.bearing) * Vec3::Z;
        tank_data.fire_direction =
            (tank_data.fire_direction * elevation.cos() + Vec3::Y * elevation.sin()).normalize();

        let gizmo_origin = tank_transform.translation + Vec3::Y * 0.3;
        let gizmo_fire_src = gizmo_origin + tank_data.fire_direction * GIZMO_EMPTY_RADIUS;
        let gizmo_fire_end =
            gizmo_origin + tank_data.fire_direction * (GIZMO_FIRE_LEN + GIZMO_EMPTY_RADIUS);
        let gizmo_blue_proj_src = Vec3::new(gizmo_fire_src.x, gizmo_origin.y, gizmo_fire_src.z);
        let gizmo_blue_proj_end = Vec3::new(gizmo_fire_end.x, gizmo_origin.y, gizmo_fire_end.z);
        tank_data.fire_origin = gizmo_fire_src;

        gizmos.line(gizmo_fire_src, gizmo_fire_end, Color::RED);
        gizmos.line(gizmo_blue_proj_src, gizmo_blue_proj_end, Color::BLUE);
        // gizmo_config.line_width = 7.0;
    }
}

fn rand_float(max_abs: f32) -> f32 {
    max_abs * (rand::random::<f32>() * 2.0 - 1.0)
}

fn rand_vec3(max_abs: f32) -> Vec3 {
    Vec3::new(
        rand_float(max_abs),
        rand_float(max_abs),
        rand_float(max_abs),
    )
}

fn tank_setup(mut commands: Commands, scene_assets: Res<GameSceneAssets>) {
    let collider_size: f32 = 1.0;
    let tank_collider = Collider::cuboid(collider_size, collider_size / 2.0, collider_size);

    let tank_controller = KinematicCharacterController {
        offset: CharacterLength::Absolute(0.01),
        max_slope_climb_angle: 25.0_f32.to_radians(),
        min_slope_slide_angle: 25.0_f32.to_radians(),
        snap_to_ground: Some(CharacterLength::Absolute(5.5)),
        ..default()
    };

    let tank_model_scene = scene_assets
        .scenes
        .get("3d/ORIGINAL/Tanks and Armored Vehicle.glb")
        .expect("KEY NOT FOUND");

    const TANK_SPAWN_COUNT: i32 = 12;
    const TANK_SPAWN_POS_MAX_SPREAD: f32 = 6000.0;
    const TANK_SPAWN_POS_MIN_SPREAD: f32 = 2000.0;
    let mut added_positions: Vec<Vec3> = vec![];

    for i in 0..TANK_SPAWN_COUNT {
        let get_pos = || {
            apply_height(&rand_vec3(TANK_SPAWN_POS_MAX_SPREAD)) + Vec3::Y * (collider_size + 1.0)
        };
        let mut tank_spawn_pos = get_pos();
        while !added_positions.is_empty()
            && added_positions
                .iter()
                .any(|other| other.distance(tank_spawn_pos) < TANK_SPAWN_POS_MIN_SPREAD)
        {
            tank_spawn_pos = get_pos();
        }
        added_positions.push(tank_spawn_pos);

        let tank_model = SceneBundle {
            scene: tank_model_scene.clone(),
            ..Default::default()
        };

        let tank_id = commands
            .spawn((Tank::default(), SpatialBundle::default()))
            .insert(GravityScale(GRAVITY_SCALE))
            .insert(ColliderMassProperties::Density(TANK_DENSITY))
            .insert(Transform::from_translation(tank_spawn_pos))
            .insert(TankGravity::default())
            .insert((
                RigidBody::KinematicPositionBased,
                tank_controller.clone(),
                tank_collider.clone(), // tank_model,
            ))
            .insert(TerrainSplitProbe)
            .id();
        commands
            .spawn((tank_model, Name::new("Tank Model")))
            .insert(
                Transform::from_translation(Vec3::Y * -0.25_f32).with_scale(Vec3::ONE * 0.25_f32),
            )
            .set_parent(tank_id); //.insert(Transform::from_scale(Vec3::ONE * 0.25));

        if i == 0 {
            commands
                .entity(tank_id)
                .insert(PlayerControlledTank)
                .insert(Name::new(format!("Player Tank ({})", i)));
        } else {
            commands
                .entity(tank_id)
                .insert(super::tank_ai::AiControlledTank::new())
                .insert(Name::new(format!("AI Tank ({})", i)));
        }
    }
}

fn tank_gravity_update(
    mut tanks: Query<(&mut KinematicCharacterController, &mut TankGravity), With<TankGravity>>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (mut tank_transform, mut gravity) in tanks.iter_mut() {
        if !gravity.is_grounded {
            gravity.fall_time += dt;
            tank_transform.translation = Some(
                tank_transform.translation.unwrap_or_default()
                    + -Vec3::Y * gravity.fall_time * GRAVITY_SCALE * GRAVITY_MAGNITUDE,
            );
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
        const RESET_BELOW: f32 = 5.0;
        let terrain_height = height(&transform.translation);
        if transform.translation.y < terrain_height - RESET_BELOW {
            warn!(
                "SHIT FELL UNDER TERRAIN: height={}  Y={}",
                terrain_height, transform.translation.y
            );
            transform.translation.y = terrain_height + RESET_BELOW * 3.;
        }
    }
}
