use bevy::prelude::*;
use crate::components::{Bird, Insect, Predator, Velocity};
use crate::spatial::SpatialIndex;

pub fn predator_sight_system(
    _spatial: Res<SpatialIndex>,
    _birds: Query<(&Transform, &mut Velocity, &Predator), With<Bird>>,
    _insects: Query<(Entity, &Transform), With<Insect>>,
) {
    // NOT IMPLEMENTED
}
