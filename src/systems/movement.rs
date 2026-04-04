use bevy::prelude::*;
use crate::components::Velocity;

pub fn movement_system(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (velocity, mut transform) in &mut query {
        transform.translation += velocity.0 * dt;
    }
}
