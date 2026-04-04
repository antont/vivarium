use bevy::prelude::*;
use crate::components::{Bird, Insect};
use crate::config::Config;
use crate::spatial::SpatialIndex;

pub fn rebuild_spatial_index(
    mut spatial: ResMut<SpatialIndex>,
    query: Query<(Entity, &Transform), Or<(With<Insect>, With<Bird>)>>,
) {
    *spatial = SpatialIndex::new(Config::SPATIAL_CELL_SIZE);
    for (entity, transform) in &query {
        spatial.insert(entity, transform.translation);
    }
}
