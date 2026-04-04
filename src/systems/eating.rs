use bevy::prelude::*;
use std::collections::HashSet;
use crate::components::{Bird, Insect, Velocity};
use crate::config::Config;
use crate::spatial::SpatialIndex;

pub fn eating_system(
    mut commands: Commands,
    spatial: Res<SpatialIndex>,
    birds: Query<&Transform, With<Bird>>,
    insects: Query<(Entity, &Transform), With<Insect>>,
) {
    let mut eaten: HashSet<Entity> = HashSet::new();

    for bird_transform in &birds {
        let bird_pos = bird_transform.translation;
        let nearby = spatial.get_nearby(bird_pos);

        for &nearby_entity in &nearby {
            if eaten.contains(&nearby_entity) {
                continue;
            }
            let Ok((insect_entity, insect_transform)) = insects.get(nearby_entity) else {
                continue;
            };

            let dist = (insect_transform.translation - bird_pos).length();
            if dist < Config::BIRD_EATING_DISTANCE {
                commands.entity(insect_entity).despawn();
                eaten.insert(insect_entity);
            }
        }
    }
}
