use bevy::prelude::*;
use crate::components::{Insect, SwarmCohesion, Velocity};
use crate::spatial::SpatialIndex;

pub fn swarm_cohesion_system(
    spatial: Res<SpatialIndex>,
    mut insects: Query<(Entity, &Transform, &mut Velocity, &SwarmCohesion), With<Insect>>,
) {
    // Snapshot positions
    let snapshots: Vec<(Entity, Vec3)> = insects
        .iter()
        .map(|(e, t, _, _)| (e, t.translation))
        .collect();

    for (entity, transform, mut velocity, cohesion) in &mut insects {
        let pos = transform.translation;
        let mut center = Vec3::ZERO;
        let mut count = 0u32;

        let nearby = spatial.get_nearby(pos);
        for &nearby_entity in &nearby {
            if nearby_entity == entity {
                continue;
            }
            let Some(&(_, other_pos)) = snapshots.iter().find(|(e, _)| *e == nearby_entity) else {
                continue;
            };
            let dist = (other_pos - pos).length();
            if dist > cohesion.radius || dist < f32::EPSILON {
                continue;
            }
            center += other_pos;
            count += 1;
        }

        if count > 0 {
            center /= count as f32;
            let toward_center = (center - pos).normalize_or_zero();
            let speed = velocity.0.length();
            velocity.0 = (velocity.0 + toward_center * cohesion.weight).normalize_or_zero() * speed;
        }
    }
}
