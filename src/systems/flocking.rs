use bevy::prelude::*;
use bevy::ecs::entity::EntityHashMap;
use crate::components::{Bird, Flocking, Velocity};
use crate::config::Config;
use crate::spatial::SpatialIndex;

pub fn flocking_system(
    spatial: Res<SpatialIndex>,
    mut birds: Query<(Entity, &Transform, &mut Velocity, &Flocking), With<Bird>>,
) {
    let snapshots: EntityHashMap<(Vec3, Vec3)> = birds
        .iter()
        .map(|(e, t, v, _)| (e, (t.translation, v.0)))
        .collect();

    birds.par_iter_mut().for_each(|(entity, transform, mut velocity, flocking)| {
        let pos = transform.translation;
        let mut separation = Vec3::ZERO;
        let mut alignment = Vec3::ZERO;
        let mut cohesion = Vec3::ZERO;
        let mut neighbor_count = 0u32;

        spatial.for_each_nearby(pos, |nearby_entity| {
            if nearby_entity == entity {
                return;
            }
            let Some(&(other_pos, other_vel)) = snapshots.get(&nearby_entity) else {
                return;
            };

            let diff = pos - other_pos;
            let dist = diff.length();
            if dist > Config::FLOCK_NEIGHBOR_RADIUS || dist < f32::EPSILON {
                return;
            }

            neighbor_count += 1;

            if dist < Config::SEPARATION_DISTANCE {
                separation += diff.normalize() / dist;
            }

            alignment += other_vel;
            cohesion += other_pos;
        });

        if neighbor_count > 0 {
            let n = neighbor_count as f32;

            alignment = alignment / n;
            let alignment_steer = alignment - velocity.0;

            cohesion = cohesion / n;
            let cohesion_steer = cohesion - pos;

            let steering = separation * flocking.separation_weight
                + alignment_steer * flocking.alignment_weight
                + cohesion_steer.normalize_or_zero() * flocking.cohesion_weight;

            let new_vel = velocity.0 + steering;
            velocity.0 = new_vel.normalize_or_zero() * Config::BIRD_SPEED;
        }
    });
}
