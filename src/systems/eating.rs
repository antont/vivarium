use bevy::prelude::*;
use crate::components::{Bird, Insect, Velocity};
use crate::config::Config;
use crate::spatial::SpatialIndex;

pub fn eating_system(
    mut commands: Commands,
    spatial: Res<SpatialIndex>,
    birds: Query<(&Transform, &Velocity), With<Bird>>,
    insects: Query<(Entity, &Transform), With<Insect>>,
) {
    for (bird_transform, _) in &birds {
        let bird_pos = bird_transform.translation;
        let nearby = spatial.get_nearby(bird_pos);

        for &nearby_entity in &nearby {
            let Ok((insect_entity, insect_transform)) = insects.get(nearby_entity) else {
                continue;
            };

            let dist = (insect_transform.translation - bird_pos).length();
            if dist < Config::BIRD_EATING_DISTANCE {
                commands.entity(insect_entity).despawn();
            }
        }
    }
}

