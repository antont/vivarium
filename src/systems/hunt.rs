use bevy::prelude::*;
use crate::components::{Bird, HuntState, Insect, Predator, Velocity, Wander};
use crate::spatial::SpatialIndex;

pub fn hunt_system(
    _time: Res<Time>,
    _spatial: Res<SpatialIndex>,
    _birds: Query<(&Transform, &mut Velocity, &Predator, &mut HuntState, &Wander), With<Bird>>,
    _insects: Query<(Entity, &Transform), With<Insect>>,
) {
    // NOT IMPLEMENTED
}
