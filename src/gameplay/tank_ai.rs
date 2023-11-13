use std::f32::consts::PI;

use bevy::{prelude::*, time::Stopwatch};
use bevy_spatial::{kdtree::KDTree3, SpatialAccess};
use rand::{random, seq::SliceRandom};

use crate::utils::cap_2pi;

use super::{events::TankCommandEvent, tank::Tank};

pub struct TankAiPlugin;
impl Plugin for TankAiPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AiControlledTank>()
            .add_systems(PreUpdate, tank_ai_progress_stopwatches)
            .add_systems(PostUpdate, tank_auto_aim)
            .add_systems(PostUpdate, tank_auto_fire)
            .add_systems(PostUpdate, tank_auto_move);
    }
}

#[derive(Reflect, Component, Default)]
pub struct AiControlledTank {
    since_fire: Stopwatch,
    fire_jitter: f32,
    since_move: Stopwatch,
    since_aim: Stopwatch,
    since_target_switch: Stopwatch,
    since_target_switch_jitter: f32,
    aim_jitter: f32,
    target: Option<Entity>,
    mvmt_ang_offset: f32,
    since_change_move_ang: Stopwatch,
}

impl AiControlledTank {
    pub fn new() -> Self {
        Self {
            fire_jitter: (random::<f32>() * 2.0 - 1.0) * AI_FIRE_JITTER,
            mvmt_ang_offset: random::<f32>() * 2.0 * PI,
            ..Default::default()
        }
    }
}

fn tank_ai_progress_stopwatches(mut tanks: Query<&mut AiControlledTank>, time: Res<Time>) {
    for mut tank in tanks.iter_mut() {
        tank.since_fire.tick(time.delta());
        tank.since_move.tick(time.delta());
        tank.since_aim.tick(time.delta());
        tank.since_target_switch.tick(time.delta());
        tank.since_change_move_ang.tick(time.delta());
    }
}

const AI_RELOAD_TIME: f32 = 3.6;
const AI_AIM_INTERVAL: f32 = 1.0;
const AI_TARGET_SWITCH_INTERVAL: f32 = 15.0;
const AI_FIRE_JITTER: f32 = AI_RELOAD_TIME * 0.2;
const AI_AIM_JITTER: f32 = AI_AIM_INTERVAL * 0.2;
const AI_TANK_MIN_SWAP_MVMT_INTERVAL: f32 = 2.0;
const AI_TANK_MAX_SWAP_MVMT_INTERVAL: f32 = 20.0;

fn tank_auto_fire(
    mut tanks: Query<(Entity, &mut AiControlledTank)>,
    mut events: EventWriter<TankCommandEvent>,
) {
    for (tank_entity, mut tank) in tanks.iter_mut() {
        if tank.since_fire.elapsed_secs() < AI_RELOAD_TIME + tank.fire_jitter {
            continue;
        }
        if tank.target.is_none() {
            continue;
        }
        let event_type = super::events::TankCommandEventType::Fire;
        events.send(TankCommandEvent {
            tank_entity,
            event_type,
        });
        tank.since_fire.reset();
        tank.fire_jitter = (random::<f32>() * 2.0 - 1.0) * AI_FIRE_JITTER;
    }
}

fn tank_auto_aim(
    mut ai_tanks: Query<(Entity, &mut AiControlledTank, &Tank, &GlobalTransform)>,
    mut events: EventWriter<TankCommandEvent>,
    potential_targets: Query<(Entity, &Tank), With<Tank>>,
    target_tank_tree: Res<KDTree3<Tank>>,
) {
    for (ai_tank_entity, mut ai_tank, ai_tank_common, ai_transform) in ai_tanks.iter_mut() {
        if ai_tank.since_aim.elapsed_secs() < AI_AIM_INTERVAL + ai_tank.aim_jitter {
            continue;
        }

        // if we had a previous target
        if let Some(target_ent) = ai_tank.target {
            // if the old target is still a valid one
            if potential_targets.contains(target_ent)
                && ai_tank.since_target_switch.elapsed_secs()
                    < AI_TARGET_SWITCH_INTERVAL + ai_tank.since_target_switch_jitter
            {
                // if the previous aim event was successful
                if let Some(solution) = ai_tank_common.fire_solutions.clone() {
                    if solution.err_sol.is_none() {
                        // aim at it again

                        let travel_time = solution.low_sol.expect("wtf?").flight_time;

                        let target_position = potential_targets
                            .get(target_ent)
                            .expect("wtf?")
                            .1
                            .estimate_future_position(travel_time);
                        let event_type =
                            super::events::TankCommandEventType::AimAtPoint(target_position);
                        events.send(TankCommandEvent {
                            tank_entity: ai_tank_entity,
                            event_type,
                        });
                        ai_tank.since_aim.reset();
                        ai_tank.aim_jitter = (random::<f32>() * 2.0 - 1.0) * AI_AIM_JITTER;
                        continue;
                    }
                }
            }
        }

        // aim at a random target
        ai_tank.since_target_switch.reset();
        ai_tank.since_target_switch_jitter = rand::random::<f32>() * AI_TARGET_SWITCH_INTERVAL;
        ai_tank.target = None;
        let our_location = ai_transform.translation();
        let mut targets = target_tank_tree
            .k_nearest_neighbour(our_location, 5)
            .iter()
            .map(|x| (x.0, x.1.unwrap()))
            .filter(|x| x.1 != ai_tank_entity)
            .collect::<Vec<_>>();
        if targets.is_empty() {
            // nothing to shoot at - we won!
            continue;
        }
        targets.shuffle(&mut rand::thread_rng());
        let (target_position, target_ent) = targets[0];
        ai_tank.target = Some(target_ent);

        let event_type = super::events::TankCommandEventType::AimAtPoint(target_position);
        events.send(TankCommandEvent {
            tank_entity: ai_tank_entity,
            event_type,
        });
        ai_tank.since_aim.reset();
        ai_tank.aim_jitter = (random::<f32>() * 2.0 - 1.0) * AI_AIM_JITTER;
    }
}

fn tank_auto_move(
    mut ai_tanks: Query<(Entity, &mut AiControlledTank, &Tank)>,
    mut events: EventWriter<TankCommandEvent>,
) {
    for (ai_tank_entity, mut ai_tank, ai_tank_common) in ai_tanks.iter_mut() {
        if ai_tank.since_change_move_ang.elapsed_secs() > AI_TANK_MAX_SWAP_MVMT_INTERVAL
            || (ai_tank.since_change_move_ang.elapsed_secs() > AI_TANK_MIN_SWAP_MVMT_INTERVAL
                && (ai_tank_common
                    .last_positions
                    .iter()
                    .map(|x| x.0)
                    .collect::<Vec<_>>()
                    .windows(2)
                    .map(|a| a[0].distance(a[1]))
                    .sum::<f32>()
                    < 0.2))
        {
            ai_tank.since_change_move_ang.reset();
            ai_tank.mvmt_ang_offset = random::<f32>() * 2.0 * PI;
        }

        events.send(TankCommandEvent {
            event_type: super::events::TankCommandEventType::MoveForward,
            tank_entity: ai_tank_entity,
        });

        let ang_diff = cap_2pi(
            ai_tank_common.body_orientation - ai_tank_common.bearing + ai_tank.mvmt_ang_offset,
        );
        const ANG_ALLOW_ERR: f32 = 0.3;
        if ang_diff > ANG_ALLOW_ERR {
            events.send(TankCommandEvent {
                event_type: super::events::TankCommandEventType::MoveLeft,
                tank_entity: ai_tank_entity,
            });
        } else if ang_diff < -ANG_ALLOW_ERR {
            events.send(TankCommandEvent {
                event_type: super::events::TankCommandEventType::MoveRight,
                tank_entity: ai_tank_entity,
            });
        }
    }
}
