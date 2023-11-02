use bevy::prelude::*;

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
