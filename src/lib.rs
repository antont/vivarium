pub mod components;
pub mod config;
pub mod spatial;
pub mod systems;

use bevy::prelude::*;
use rand::Rng;

use components::*;
use config::Config;

/// Spawn insect swarm entities. Returns the number spawned.
pub fn spawn_insects(world: &mut World) -> usize {
    let mut rng = rand::rng();
    let half = Config::WORLD_HALF_SIZE;

    for _ in 0..Config::INSECT_COUNT {
        let position = Vec3::new(
            rng.random_range(-half..half),
            rng.random_range(-half..half),
            rng.random_range(-half..half),
        );
        let direction = Vec3::new(
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
        )
        .normalize_or_zero();

        world.spawn((
            Insect,
            Transform::from_translation(position),
            Velocity(direction * Config::INSECT_SPEED),
            BrownianMotion {
                wander_strength: Config::INSECT_WANDER_STRENGTH,
            },
            BoundaryWrap,
        ));
    }

    Config::INSECT_COUNT
}

/// Spawn bird entities. Returns the number spawned.
pub fn spawn_birds(world: &mut World) -> usize {
    let mut rng = rand::rng();
    let half = Config::WORLD_HALF_SIZE;

    for _ in 0..Config::BIRD_COUNT {
        let position = Vec3::new(
            rng.random_range(-half * 0.3..half * 0.3),
            rng.random_range(-half * 0.3..half * 0.3),
            rng.random_range(-half * 0.3..half * 0.3),
        );
        let direction = Vec3::new(
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
        )
        .normalize_or_zero();

        world.spawn((
            Bird,
            Transform::from_translation(position),
            Velocity(direction * Config::BIRD_SPEED),
            Predator {
                sight_range: Config::BIRD_SIGHT_RANGE,
                sight_half_angle: Config::BIRD_SIGHT_HALF_ANGLE,
            },
            Flocking {
                separation_weight: Config::SEPARATION_WEIGHT,
                alignment_weight: Config::ALIGNMENT_WEIGHT,
                cohesion_weight: Config::COHESION_WEIGHT,
            },
            BoundaryWrap,
        ));
    }

    Config::BIRD_COUNT
}
