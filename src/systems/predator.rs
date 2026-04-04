use bevy::prelude::*;
use crate::components::{Bird, Insect, Predator, Velocity};
use crate::spatial::SpatialIndex;

pub fn predator_sight_system(
    spatial: Res<SpatialIndex>,
    mut birds: Query<(&Transform, &mut Velocity, &Predator), With<Bird>>,
    insects: Query<(Entity, &Transform), With<Insect>>,
) {
    for (bird_transform, mut velocity, predator) in &mut birds {
        let bird_pos = bird_transform.translation;
        let forward = velocity.0.normalize_or_zero();
        if forward == Vec3::ZERO {
            continue;
        }

        let mut closest_dist = f32::MAX;
        let mut closest_dir = None;

        let nearby = spatial.get_nearby(bird_pos);
        for &nearby_entity in &nearby {
            let Ok((_, insect_transform)) = insects.get(nearby_entity) else {
                continue;
            };

            let to_insect = insect_transform.translation - bird_pos;
            let dist = to_insect.length();

            if dist > predator.sight_range || dist < f32::EPSILON {
                continue;
            }

            let to_insect_norm = to_insect / dist;
            let angle = forward.dot(to_insect_norm).clamp(-1.0, 1.0).acos();

            if angle < predator.sight_half_angle && dist < closest_dist {
                closest_dist = dist;
                closest_dir = Some(to_insect_norm);
            }
        }

        if let Some(target_dir) = closest_dir {
            let steer_strength = 3.0;
            let new_vel = velocity.0 + target_dir * steer_strength;
            let speed = velocity.0.length();
            velocity.0 = new_vel.normalize_or_zero() * speed;
        }
    }
}
