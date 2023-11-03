use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TankCommandEventType {
    PowerPlus,
    PowerMinus,
    ElevationPlus,
    ElevationMinus,
    BearingRight,
    BearingLeft,
    MoveForward,
    MoveBack,
    MoveLeft,
    MoveRight,

    AimAtPoint(Vec3),
    Fire,
}

#[derive(Event, Debug, Copy, Clone, PartialEq)]
pub struct TankCommandEvent {
    pub event_type: TankCommandEventType,
    pub tank_entity: Entity,
}

#[derive(Reflect, Event, Debug, Clone)]
pub struct BulletHitEvent {
    pub bullet_vel: Velocity,
    pub bullet_pos: Vec3,
    pub directly_hit_ent: Entity,
    pub tank_ent: Entity,
}
