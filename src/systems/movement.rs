use bevy::prelude::*;
use crate::components::Velocity;

pub fn movement_system(
    _time: Res<Time>,
    _query: Query<(&Velocity, &mut Transform)>,
) {
    // NOT IMPLEMENTED
}
