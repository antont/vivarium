use bevy::prelude::*;
use crate::components::{Bird, Velocity};

/// Rotates bird entities to face their velocity direction.
pub fn face_velocity_system(
    mut query: Query<(&Velocity, &mut Transform), With<Bird>>,
) {
    for (velocity, mut transform) in &mut query {
        let dir = velocity.0.normalize_or_zero();
        if dir != Vec3::ZERO {
            // look_at points -Z toward target, so look away from velocity
            // to make the +Z nose point forward
            let look_target = transform.translation - dir;
            transform.look_at(look_target, Vec3::Y);
        }
    }
}
