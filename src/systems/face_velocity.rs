use crate::components::{Bird, Velocity};
use bevy::prelude::*;

/// Rotates bird entities so the cone tip points in the velocity direction.
pub fn face_velocity_system(mut query: Query<(&Velocity, &mut Transform), With<Bird>>) {
    for (velocity, mut transform) in &mut query {
        let dir = velocity.0.normalize_or_zero();
        if dir != Vec3::ZERO {
            // Custom bird mesh points along +Z by default (beak/head at +Z).
            // Compute rotation from +Z to the velocity direction.
            transform.rotation = Quat::from_rotation_arc(Vec3::Z, dir);
        }
    }
}
