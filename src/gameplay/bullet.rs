use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::gameplay::bullet_physics::{BULLET_DENSITY, BULLET_LINEAR_DAMPING, GRAVITY_SCALE};

use crate::planet::TerrainSplitProbe;
use crate::{game_assets::BulletAssets, gameplay::events::TankCommandEventType};

use super::{
    bullet_physics::TANK_BULLET_SPEED_PER_POWER,
    events::TankCommandEvent,
    tank::{PlayerControlledTank, Tank},
};

pub struct BulletPlugin;
impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (shoot_bullet, capture_bullet_impact).chain())
            .add_systems(PostUpdate, on_bullet_impact);
    }
}

#[derive(Reflect, Component, Default)]
pub struct Bullet;

#[derive(Reflect, Component)]
pub struct BulletHit {
    other_thing_hit: Entity,
    // hit_location: Vec3,
}

fn on_bullet_impact(mut commands: Commands, hits: Query<(Entity, &mut Bullet, &mut BulletHit)>) {
    for (bullet_ent, _bullet, _bullet_hit) in hits.iter() {
        // trigger some events and shit
        // TODO

        // finally, delete the bullet
        commands.entity(bullet_ent).despawn_recursive();
    }
}

fn shoot_bullet(
    mut commands: Commands,
    player_tank: Query<(Entity, &Tank), With<PlayerControlledTank>>,
    bullet_assets: Res<BulletAssets>,
    // scene_assets: ResMut<GameSceneAssets>,
    mut events: EventReader<TankCommandEvent>,
) {
    if let Ok((tank_entity, tank)) = player_tank.get_single() {
        let have_fire_event = events
            .iter()
            .any(|e| e.event_type == TankCommandEventType::Fire && e.tank_entity == tank_entity);
        if !have_fire_event {
            return;
        }

        const SHOOT_ROTATION: f32 = 5.0;

        let fwd = tank.fire_direction.normalize();
        let quat = Quat::from_rotation_arc(Vec3::Z, fwd);
        let spawn_pos = tank.fire_origin;

        let bullet_bundle = PbrBundle {
            mesh: bullet_assets.mesh.clone(),
            material: bullet_assets.material.clone(),
            transform: Transform::from_translation(spawn_pos)
                .with_rotation(quat)
                .with_scale(Vec3::ONE * 0.2),
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

        let bullet_id = commands
            .spawn((Bullet, bullet_bundle))
            .insert(RigidBody::Dynamic)
            .insert(GravityScale(GRAVITY_SCALE))
            .insert(ColliderMassProperties::Density(BULLET_DENSITY))
            .insert(bullet_assets.collider.clone())
            .insert(Ccd::enabled())
            .insert(Damping {
                linear_damping: BULLET_LINEAR_DAMPING,
                angular_damping: BULLET_LINEAR_DAMPING,
            })
            // .insert(ExternalImpulse {
            //     impulse: fwd * tank.power * SHOOT_IMPULSE_SCALE,
            //     torque_impulse: quat * Vec3::new(0.0, 0.0, SHOOT_ROTATION),
            // })
            .insert(Velocity {
                linvel: fwd * tank.power * TANK_BULLET_SPEED_PER_POWER,
                angvel: quat * Vec3::new(0.0, 0.0, SHOOT_ROTATION),
            })
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Name::new("BULLET"))
            .insert(TerrainSplitProbe)
            .id();

        commands
            .spawn(bevy_hanabi::prelude::ParticleEffectBundle {
                effect: bevy_hanabi::ParticleEffect::new(bullet_assets.flying_effect.clone()),
                ..Default::default()
            })
            .set_parent(bullet_id);
    }
}

fn capture_bullet_impact(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    bullet_query: Query<Entity, With<Bullet>>,
) {
    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(col1, col2, _flags) = collision_event {
            if bullet_query.contains(col1.clone()) {
                commands.entity(col1.clone()).insert(BulletHit {
                    other_thing_hit: col2.clone(),
                });
            }
            if bullet_query.contains(col2.clone()) {
                commands.entity(col2.clone()).insert(BulletHit {
                    other_thing_hit: col1.clone(),
                });
            }
        }
    }
}
