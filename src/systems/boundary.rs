use bevy::prelude::*;
use crate::components::{BoundaryWrap, Velocity};
use crate::config::{Config, SimScale};

pub fn boundary_force_system(
    mut query: Query<(&mut Transform, &mut Velocity), With<BoundaryWrap>>,
    scale: Res<SimScale>,
) {
    let half = scale.size(Config::WORLD_HALF_SIZE);
    let margin = Config::BOUNDARY_MARGIN;
    let strength = Config::BOUNDARY_FORCE;
    let inner = half - margin;

    for (mut transform, mut velocity) in &mut query {
        let pos = transform.translation;

        let mut steer = Vec3::ZERO;

        // Per-axis repulsion toward center, quadratic ramp near edges
        for i in 0..3 {
            let p = pos[i];
            if p > inner {
                let t = ((p - inner) / margin).min(1.0);
                steer[i] -= strength * t * t;
            } else if p < -inner {
                let t = ((-inner - p) / margin).min(1.0);
                steer[i] += strength * t * t;
            }
        }

        if steer != Vec3::ZERO {
            // Apply force additively — this changes both direction AND magnitude,
            // creating a genuine repulsion effect rather than just steering
            velocity.0 += steer;
        }

        // Hard clamp: if somehow already outside, push back in
        transform.translation = pos.clamp(Vec3::splat(-half), Vec3::splat(half));
    }
}
