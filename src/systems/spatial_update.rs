use bevy::prelude::*;
use crate::components::{Bird, Insect};
use crate::spatial::SpatialIndex;

pub fn rebuild_spatial_index(
    _spatial: ResMut<SpatialIndex>,
    _query: Query<(Entity, &Transform), Or<(With<Insect>, With<Bird>)>>,
) {
    // NOT IMPLEMENTED
}
