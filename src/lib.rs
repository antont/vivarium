pub mod components;
pub mod config;
pub mod lsystem_tree;
pub mod nav_graph;
pub mod orbit_camera;
pub mod spatial;
pub mod squirrel;
pub mod systems;
pub mod wind;

use bevy::prelude::*;
use rand::Rng;

use components::*;
use config::Config;

/// Plugin that registers all vivarium simulation systems, resources, and messages.
/// Add this plugin to any App (main, examples, tests) to get the full simulation.
/// You still need to provide your own Startup system to spawn entities.
pub struct VivariumPlugin;

impl Plugin for VivariumPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(spatial::SpatialIndex::new(Config::SPATIAL_CELL_SIZE))
            .insert_resource(wind::Wind::default())
            .insert_resource(orbit_camera::CameraMode::default())
            .add_message::<InsectEaten>()
            .add_systems(
                Update,
                (
                    wind::wind_update_system,
                    systems::spatial_update::rebuild_spatial_index,
                    (
                        systems::brownian::brownian_motion_system,
                        systems::swarm_cohesion::swarm_cohesion_system,
                        systems::flocking::flocking_system,
                    ),
                    systems::hunt::hunt_system,
                    systems::nesting::bird_fly_to_target_system,
                    systems::movement::movement_system,
                    wind::wind_tree_system,
                    systems::face_velocity::face_velocity_system,
                    systems::eating::eating_system,
                    systems::nesting::bird_lifecycle_system,
                    systems::nesting::hatchling_alert_system,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    orbit_camera::camera_mode_system,
                    orbit_camera::orbit_camera_system,
                    squirrel::squirrel_hatchling_detection_system,
                    squirrel::squirrel_behavior_system,
                    squirrel::squirrel_movement_system,
                    squirrel::squirrel_flee_system,
                    systems::nesting::nest_visual_system,
                    systems::nesting::hatchling_visual_system,
                ),
            )
            .add_systems(PostUpdate, systems::boundary::boundary_force_system);
    }
}

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
