use bevy::prelude::*;
use rand::Rng;
use crate::components::{BrownianMotion, Insect, Velocity};
use crate::config::Config;

pub fn brownian_motion_system(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &BrownianMotion), With<Insect>>,
) {
    let dt = time.delta_secs();
    let mut rng = rand::rng();

    for (mut velocity, brownian) in &mut query {
        let perturbation = Vec3::new(
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
        ) * brownian.wander_strength * dt;

        let new_vel = velocity.0 + perturbation;
        velocity.0 = new_vel.normalize_or_zero() * Config::INSECT_SPEED;
    }
}
