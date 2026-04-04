use bevy::prelude::*;
use crate::components::{Bird, Flocking, Velocity};
use crate::config::Config;
use crate::spatial::SpatialIndex;

pub fn flocking_system(
    spatial: Res<SpatialIndex>,
    mut birds: Query<(Entity, &Transform, &mut Velocity, &Flocking), With<Bird>>,
) {
    // Phase 1: snapshot all bird data
    let snapshots: Vec<(Entity, Vec3, Vec3)> = birds
        .iter()
        .map(|(e, t, v, _)| (e, t.translation, v.0))
        .collect();

    // Phase 2: compute and apply steering for each bird
    for (entity, transform, mut velocity, flocking) in &mut birds {
        let pos = transform.translation;
        let mut separation = Vec3::ZERO;
        let mut alignment = Vec3::ZERO;
        let mut cohesion = Vec3::ZERO;
        let mut neighbor_count = 0u32;

        let nearby = spatial.get_nearby(pos);
        for &nearby_entity in &nearby {
            if nearby_entity == entity {
                continue;
            }
            // Find this entity in snapshots
            let Some(&(_, other_pos, other_vel)) = snapshots
                .iter()
                .find(|(e, _, _)| *e == nearby_entity)
            else {
                continue;
            };

            let diff = pos - other_pos;
            let dist = diff.length();
            if dist > Config::FLOCK_NEIGHBOR_RADIUS || dist < f32::EPSILON {
                continue;
            }

            neighbor_count += 1;

            // Separation: steer away from close neighbors
            if dist < Config::SEPARATION_DISTANCE {
                separation += diff.normalize() / dist;
            }

            // Alignment: match heading
            alignment += other_vel;

            // Cohesion: steer toward center of mass
            cohesion += other_pos;
        }

        if neighbor_count > 0 {
            let n = neighbor_count as f32;

            // Alignment: average heading of neighbors
            alignment = alignment / n;
            let alignment_steer = alignment - velocity.0;

            // Cohesion: direction toward center of mass
            cohesion = cohesion / n;
            let cohesion_steer = cohesion - pos;

            let steering = separation * flocking.separation_weight
                + alignment_steer * flocking.alignment_weight
                + cohesion_steer.normalize_or_zero() * flocking.cohesion_weight;

            let new_vel = velocity.0 + steering;
            velocity.0 = new_vel.normalize_or_zero() * Config::BIRD_SPEED;
        }
    }
}
