use bevy::prelude::*;
use crate::components::{BoundaryWrap, Velocity};

pub fn boundary_force_system(
    _query: Query<(&Transform, &mut Velocity), With<BoundaryWrap>>,
) {
    // NOT IMPLEMENTED
}
