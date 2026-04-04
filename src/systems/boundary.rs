use bevy::prelude::*;
use crate::components::BoundaryWrap;

pub fn boundary_wrap_system(
    _query: Query<&mut Transform, With<BoundaryWrap>>,
) {
    // NOT IMPLEMENTED
}
