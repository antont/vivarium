use bevy::prelude::*;
use crate::components::{BrownianMotion, Insect, Velocity};

pub fn brownian_motion_system(
    _time: Res<Time>,
    _query: Query<(&mut Velocity, &BrownianMotion), With<Insect>>,
) {
    // NOT IMPLEMENTED
}
