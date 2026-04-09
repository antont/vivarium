use bevy::prelude::*;
use bevy::ecs::entity::EntityHashMap;
use crate::components::{Insect, SwarmCohesion, Velocity};
use crate::spatial::SpatialIndex;

pub fn swarm_cohesion_system(
    spatial: Res<SpatialIndex>,
    mut insects: Query<(Entity, &Transform, &mut Velocity, &SwarmCohesion), With<Insect>>,
) {
    let snapshots: EntityHashMap<Vec3> = insects
        .iter()
        .map(|(e, t, _, _)| (e, t.translation))
        .collect();

    insects.par_iter_mut().for_each(|(entity, transform, mut velocity, cohesion)| {
        let pos = transform.translation;
        let mut center = Vec3::ZERO;
        let mut count = 0u32;

        spatial.for_each_nearby(pos, |nearby_entity| {
            if nearby_entity == entity {
                return;
            }
            let Some(&other_pos) = snapshots.get(&nearby_entity) else {
                return;
            };
            let dist = (other_pos - pos).length();
            if dist > cohesion.radius || dist < f32::EPSILON {
                return;
            }
            center += other_pos;
            count += 1;
        });

        if count > 0 {
            center /= count as f32;
            let toward_center = (center - pos).normalize_or_zero();
            let speed = velocity.0.length();
            velocity.0 = (velocity.0 + toward_center * cohesion.weight).normalize_or_zero() * speed;
        }
    });
}
