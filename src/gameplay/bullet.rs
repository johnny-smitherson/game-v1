use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::flying_camera::FlyingCameraPivot;
use crate::game_assets::BulletAssets;
use crate::menu::UiMenuState;
use crate::planet::TerrainSplitProbe;

use super::tank::{PlayerControlledTank, Tank};

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
    ui_state: Res<UiMenuState>,
    player_tank: Query<&Tank, With<PlayerControlledTank>>,
    bullet_assets: Res<BulletAssets>,
    // scene_assets: ResMut<GameSceneAssets>,
    keys: Res<Input<KeyCode>>,
) {
    if !ui_state.is_mouse_captured {
        return;
    }
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }
    let tank = player_tank.get_single().expect("no player tank wtf.");

    const SHOOT_IMPULSE_SCALE: f32 = 0.3;
    const SHOOT_ROTATION: f32 = 5.0;
    const SHOOT_EXTRA_FORWARD: f32 = 1.5;

    let fwd = tank.fire_direction;
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
        .insert(ColliderMassProperties::Density(2.0))
        .insert(bullet_assets.collider.clone())
        .insert(Ccd::enabled())
        .insert(Damping {
            linear_damping: 0.05,
            angular_damping: 0.05,
        })
        .insert(ExternalImpulse {
            impulse: fwd * tank.power * SHOOT_IMPULSE_SCALE,
            torque_impulse: quat * Vec3::new(0.0, 0.0, SHOOT_ROTATION),
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
