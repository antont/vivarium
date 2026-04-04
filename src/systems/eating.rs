use bevy::prelude::*;
use crate::components::{Bird, Insect, Velocity};
use crate::spatial::SpatialIndex;

pub fn eating_system(
    _commands: Commands,
    _spatial: Res<SpatialIndex>,
    _birds: Query<(&Transform, &Velocity), With<Bird>>,
    _insects: Query<(Entity, &Transform), With<Insect>>,
) {
    // NOT IMPLEMENTED
}

pub fn insect_respawn_system(
    _commands: Commands,
    _insects: Query<&Insect>,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
) {
    // NOT IMPLEMENTED
}
