use bevy::prelude::*;
use rand::Rng;
use crate::components::{BrownianMotion, Insect, Velocity};
use crate::config::Config;

pub fn brownian_motion_system(
    _time: Res<Time>,
    mut query: Query<(&mut Velocity, &BrownianMotion), With<Insect>>,
) {
    let mut rng = rand::rng();

    for (mut velocity, brownian) in &mut query {
        // Random axis and angle for smooth directional wandering.
        // Not scaled by dt — this is a per-frame random walk on direction,
        // not a force. The wander_strength controls the max angle per frame.
        let random_axis = Vec3::new(
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
        )
        .normalize_or_zero();

        if random_axis != Vec3::ZERO {
            let angle = rng.random_range(-brownian.wander_strength..brownian.wander_strength);
            let rotation = Quat::from_axis_angle(random_axis, angle.to_radians());
            velocity.0 = (rotation * velocity.0).normalize_or_zero() * Config::INSECT_SPEED;
        }
    }
}
