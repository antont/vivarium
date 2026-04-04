use bevy::prelude::*;
use crate::components::{Bird, Flocking, Velocity};
use crate::spatial::SpatialIndex;

pub fn flocking_system(
    _spatial: Res<SpatialIndex>,
    _birds: Query<(Entity, &Transform, &mut Velocity, &Flocking), With<Bird>>,
) {
    // NOT IMPLEMENTED
}
