use crate::audio::PlaySpatialAudioEvent;
use crate::gameplay::bullet_physics::{BULLET_DENSITY, BULLET_LINEAR_DAMPING, GRAVITY_SCALE};
use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::random;

use crate::planet::TerrainSplitProbe;
use crate::{assets::BulletAssets, gameplay::events::TankCommandEventType};

use super::events::BulletHitEvent;
use super::{bullet_physics::TANK_BULLET_SPEED_PER_POWER, events::TankCommandEvent, tank::Tank};
use std::time::Duration;

pub struct BulletPlugin;
impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, (delete_tombstones,))
            .add_systems(Update, (shoot_bullet, capture_bullet_impact).chain())
            .add_systems(PostUpdate, (on_bullet_impact,));
    }
}

#[derive(Reflect, Component, Debug)]
pub struct Bullet {
    shooter: Entity,
}

#[derive(Reflect, Component, Debug)]
pub struct BulletTombstone(Timer);

#[derive(Reflect, Component, Debug)]
pub struct BulletFlyingEffectMarker;

#[derive(Reflect, Component, Debug)]
pub struct BulletExplodingEffectMarker;

#[derive(Reflect, Component)]
pub struct BulletHit {
    other_thing_hit: Entity,
    // hit_location: Vec3,
}

fn delete_tombstones(
    mut commands: Commands,
    mut q: Query<(Entity, &mut BulletTombstone)>,
    time: Res<Time>,
) {
    for (entity, mut tombstone) in q.iter_mut() {
        tombstone.0.tick(time.delta());

        if tombstone.0.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn on_bullet_impact(
    mut commands: Commands,
    hits: Query<
        (
            Entity,
            &mut Transform,
            &Velocity,
            &mut Children,
            &BulletHit,
            &Bullet,
        ),
        (With<Bullet>, With<BulletHit>),
    >,
    mut flying_effects: Query<(Entity, &mut EffectSpawner), With<BulletFlyingEffectMarker>>,
    bullet_assets: Res<BulletAssets>,
    mut events: EventWriter<BulletHitEvent>,
    mut audio_events: EventWriter<PlaySpatialAudioEvent>,
) {
    for (bullet_ent, bullet_tr, bullet_vel, bullet_children, bullet_hit, bullet) in hits.iter() {
        // send event with all data
        events.send(BulletHitEvent {
            bullet_vel: *bullet_vel,
            bullet_pos: bullet_tr.translation,
            directly_hit_ent: bullet_hit.other_thing_hit,
            tank_ent: bullet.shooter,
        });
        // put the tombstone on the thing
        let tombstone_ent = commands
            .spawn((
                BulletTombstone(Timer::new(Duration::from_secs(6), TimerMode::Once)),
                SpatialBundle::from_transform(Transform::from_translation(bullet_tr.translation)),
            ))
            .insert(Name::new("Bullet TOMBSTONE"))
            .insert(TerrainSplitProbe)
            .id();
        // move the flying effect to tombstone and stop it from emitting
        for child in bullet_children.iter() {
            if let Ok((effect_ent, mut effect)) = flying_effects.get_mut(*child) {
                commands.entity(effect_ent).set_parent(tombstone_ent);
                effect.set_active(false);
            }
        }
        // create explosion effect
        commands
            .spawn((
                BulletExplodingEffectMarker,
                ParticleEffectBundle {
                    effect: ParticleEffect::new(bullet_assets.hit_effect.clone()),
                    ..Default::default()
                },
            ))
            .set_parent(tombstone_ent);
        // create audio effects: far boom + closer explosion
        audio_events.send(PlaySpatialAudioEvent::distant_boom(tombstone_ent));
        audio_events.send(PlaySpatialAudioEvent::close_explosion(tombstone_ent));
        // finally, delete the bullet
        commands.entity(bullet_ent).despawn_recursive();
    }
}

fn shoot_bullet(
    mut commands: Commands,
    tanks: Query<(Entity, &Tank)>,
    bullet_assets: Res<BulletAssets>,
    mut events: EventReader<TankCommandEvent>,
    mut audio_events: EventWriter<PlaySpatialAudioEvent>,
) {
    for event in events.iter() {
        if let Ok((tank_entity, tank)) = tanks.get(event.tank_entity) {
            if event.event_type != TankCommandEventType::Fire {
                continue;
            }
            const SHOOT_ROTATION: f32 = 5.0;

            let fwd = tank.fire_direction.normalize();
            let quat = Quat::from_rotation_arc(Vec3::Z, fwd);
            let spawn_pos = tank.fire_origin;

            let bullet_pbr_bundle = PbrBundle {
                mesh: bullet_assets.mesh.clone(),
                material: bullet_assets.material.clone(),
                transform: Transform::from_translation(spawn_pos).with_rotation(quat),
                ..default()
            };

            // let bullet_bundle = SceneBundle {
            //     scene: scene_assets
            //         .scenes
            //         .get("ORIGINAL/Tanks and Armored Vehicle.glb")
            //         .expect("KEY NOT FOUND")
            //         .clone(),
            //     transform: Transform::from_translation(spawn_pos).with_rotation(quat),
            //     ..Default::default()
            // };

            const SHOOT_VEL_RELATIVE_ERR: f32 = 7.0 / 3000.0;
            let (err_x, err_y, err_z): (f32, f32, f32) = random();
            let linear_relative_err = Vec3::new(err_x, err_y, err_z) * 2.0 - Vec3::ONE;
            let linear_relative_err = linear_relative_err.normalize() * SHOOT_VEL_RELATIVE_ERR;
            let linear_vel = (fwd + linear_relative_err) * tank.power * TANK_BULLET_SPEED_PER_POWER;

            let bullet_id = commands
                .spawn((
                    Bullet {
                        shooter: tank_entity,
                    },
                    bullet_pbr_bundle,
                ))
                .insert(RigidBody::Dynamic)
                .insert(GravityScale(GRAVITY_SCALE))
                .insert(ColliderMassProperties::Density(BULLET_DENSITY))
                .insert(bullet_assets.collider.clone())
                .insert(Ccd::enabled())
                .insert(Damping {
                    linear_damping: BULLET_LINEAR_DAMPING,
                    angular_damping: BULLET_LINEAR_DAMPING,
                })
                .insert(Velocity {
                    linvel: linear_vel,
                    angvel: quat * Vec3::new(0.0, 0.0, SHOOT_ROTATION),
                })
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Name::new("BULLET"))
                .insert(TerrainSplitProbe)
                .id();

            // bullet particle effect
            commands
                .spawn((
                    BulletFlyingEffectMarker,
                    bevy_hanabi::prelude::ParticleEffectBundle {
                        effect: ParticleEffect::new(bullet_assets.flying_effect.clone()),
                        ..Default::default()
                    },
                ))
                .set_parent(bullet_id);

            // tank fire audio effect
            audio_events.send(PlaySpatialAudioEvent::canon_fire(tank_entity));
        }
    }
}

fn capture_bullet_impact(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    bullet_query: Query<Entity, With<Bullet>>,
) {
    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(col1, col2, _flags) = collision_event {
            if bullet_query.contains(*col1) {
                commands.entity(*col1).insert(BulletHit {
                    other_thing_hit: *col2,
                });
            }
            if bullet_query.contains(*col2) {
                commands.entity(*col2).insert(BulletHit {
                    other_thing_hit: *col1,
                });
            }
        }
    }
}
