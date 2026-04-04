use bevy::prelude::*;
use crate::components::{Insect, SwarmCohesion, Velocity};
use crate::spatial::SpatialIndex;

pub fn swarm_cohesion_system(
    _spatial: Res<SpatialIndex>,
    _insects: Query<(Entity, &Transform, &mut Velocity, &SwarmCohesion), With<Insect>>,
) {
    // NOT IMPLEMENTED
}
