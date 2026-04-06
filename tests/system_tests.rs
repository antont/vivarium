use bevy::prelude::*;
use vivarium::components::*;
use vivarium::config::Config;
use vivarium::spatial::SpatialIndex;
use vivarium::systems::boundary::boundary_force_system;
use vivarium::systems::brownian::brownian_motion_system;
use vivarium::systems::eating::eating_system;
use vivarium::systems::flocking::flocking_system;
use vivarium::systems::movement::movement_system;
use vivarium::systems::predator::predator_sight_system;
use vivarium::systems::spatial_update::rebuild_spatial_index;
use vivarium::wind::Wind;

use bevy::time::TimeUpdateStrategy;

/// Helper: create a minimal Bevy App with fixed time step for deterministic testing.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // Use a fixed time step so delta_secs is always 1/60
    app.insert_resource(TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f64(1.0 / 60.0),
    ));
    app.insert_resource(SpatialIndex::new(Config::SPATIAL_CELL_SIZE));
    app.insert_resource(Wind::default());
    app.add_message::<InsectEaten>();
    app
}

// =============================================================
// Movement system tests
// =============================================================

#[test]
fn movement_system_applies_velocity_to_transform() {
    let mut app = test_app();
    app.add_systems(Update, movement_system);

    let entity = app.world_mut().spawn((
        Velocity(Vec3::new(10.0, 0.0, 0.0)),
        Transform::from_translation(Vec3::ZERO),
    )).id();

    // First update establishes time baseline (delta=0), second has real delta
    app.update();
    app.update();

    let transform = app.world().get::<Transform>(entity).unwrap();
    // After one update with some delta_time, x should have increased
    assert!(
        transform.translation.x > 0.0,
        "Movement system should move entity in velocity direction, got x={}",
        transform.translation.x
    );
}

#[test]
fn movement_system_moves_in_correct_direction() {
    let mut app = test_app();
    app.add_systems(Update, movement_system);

    let entity = app.world_mut().spawn((
        Velocity(Vec3::new(0.0, -50.0, 0.0)),
        Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)),
    )).id();

    app.update();
    app.update();

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert!(
        transform.translation.y < 100.0,
        "Entity with negative Y velocity should move downward, got y={}",
        transform.translation.y
    );
}

#[test]
fn movement_system_does_not_move_entities_without_velocity() {
    let mut app = test_app();
    app.add_systems(Update, movement_system);

    let entity = app.world_mut().spawn(
        Transform::from_translation(Vec3::new(50.0, 50.0, 50.0)),
    ).id();

    app.update();

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_eq!(
        transform.translation,
        Vec3::new(50.0, 50.0, 50.0),
        "Entity without Velocity should not move"
    );
}

// =============================================================
// Boundary force field system tests
// =============================================================

#[test]
fn boundary_steers_entity_away_from_positive_edge() {
    let mut app = test_app();
    app.add_systems(Update, boundary_force_system);

    let half = Config::WORLD_HALF_SIZE;
    // Entity near +X edge, heading toward it
    let entity = app.world_mut().spawn((
        Transform::from_translation(Vec3::new(half - 10.0, 0.0, 0.0)),
        Velocity(Vec3::X * 30.0),
        BoundaryWrap,
    )).id();

    app.update();

    let vel = app.world().get::<Velocity>(entity).unwrap().0;
    assert!(
        vel.x < 30.0,
        "Boundary force should steer entity away from +X edge, got vx={}",
        vel.x
    );
}

#[test]
fn boundary_steers_entity_away_from_negative_edge() {
    let mut app = test_app();
    app.add_systems(Update, boundary_force_system);

    let half = Config::WORLD_HALF_SIZE;
    let entity = app.world_mut().spawn((
        Transform::from_translation(Vec3::new(0.0, -half + 10.0, 0.0)),
        Velocity(Vec3::new(0.0, -30.0, 0.0)),
        BoundaryWrap,
    )).id();

    app.update();

    let vel = app.world().get::<Velocity>(entity).unwrap().0;
    assert!(
        vel.y > -30.0,
        "Boundary force should steer entity away from -Y edge, got vy={}",
        vel.y
    );
}

#[test]
fn boundary_force_works_on_z_axis() {
    let mut app = test_app();
    app.add_systems(Update, boundary_force_system);

    let half = Config::WORLD_HALF_SIZE;
    let entity = app.world_mut().spawn((
        Transform::from_translation(Vec3::new(0.0, 0.0, half - 5.0)),
        Velocity(Vec3::Z * 30.0),
        BoundaryWrap,
    )).id();

    app.update();

    let vel = app.world().get::<Velocity>(entity).unwrap().0;
    assert!(
        vel.z < 30.0,
        "Boundary force should steer entity away from +Z edge, got vz={}",
        vel.z
    );
}

#[test]
fn boundary_force_does_not_affect_entity_near_center() {
    let mut app = test_app();
    app.add_systems(Update, boundary_force_system);

    let initial_vel = Vec3::new(10.0, -20.0, 5.0);
    let entity = app.world_mut().spawn((
        Transform::from_translation(Vec3::ZERO),
        Velocity(initial_vel),
        BoundaryWrap,
    )).id();

    app.update();

    let vel = app.world().get::<Velocity>(entity).unwrap().0;
    assert_eq!(vel, initial_vel, "Entity near center should not be affected by boundary force");
}

#[test]
fn boundary_force_does_not_affect_entity_without_marker() {
    let mut app = test_app();
    app.add_systems(Update, boundary_force_system);

    let half = Config::WORLD_HALF_SIZE;
    let initial_vel = Vec3::X * 30.0;
    let entity = app.world_mut().spawn((
        Transform::from_translation(Vec3::new(half - 5.0, 0.0, 0.0)),
        Velocity(initial_vel),
    )).id();

    app.update();

    let vel = app.world().get::<Velocity>(entity).unwrap().0;
    assert_eq!(vel, initial_vel, "Entity without BoundaryWrap should not be steered");
}

#[test]
fn boundary_hard_clamps_position_if_already_outside() {
    let mut app = test_app();
    app.add_systems(Update, boundary_force_system);

    let half = Config::WORLD_HALF_SIZE;
    let entity = app.world_mut().spawn((
        Transform::from_translation(Vec3::new(half + 50.0, 0.0, 0.0)),
        Velocity(Vec3::X * 30.0),
        BoundaryWrap,
    )).id();

    app.update();

    let pos = app.world().get::<Transform>(entity).unwrap().translation;
    assert!(
        pos.x <= half,
        "Entity outside bounds should be clamped back inside, got x={}",
        pos.x
    );
}

// =============================================================
// Brownian motion system tests
// =============================================================

#[test]
fn brownian_motion_changes_insect_velocity() {
    let mut app = test_app();
    app.add_systems(Update, brownian_motion_system);

    let initial_vel = Vec3::new(1.0, 0.0, 0.0).normalize() * Config::INSECT_SPEED;
    let entity = app.world_mut().spawn((
        Insect,
        Velocity(initial_vel),
        BrownianMotion {
            wander_strength: 50.0, // high strength for test to see clear direction change
        },
    )).id();

    // Run several updates to accumulate random perturbations
    for _ in 0..20 {
        app.update();
    }

    let vel = app.world().get::<Velocity>(entity).unwrap().0;
    // After 20 updates with strong wander, velocity should have deviated from pure +X
    let angle = vel.normalize().dot(Vec3::X).acos();
    assert!(
        angle > 0.01,
        "After 20 updates, brownian motion should have changed direction. Angle from +X: {}",
        angle
    );
}

#[test]
fn brownian_motion_maintains_speed() {
    let mut app = test_app();
    app.add_systems(Update, brownian_motion_system);

    let entity = app.world_mut().spawn((
        Insect,
        Velocity(Vec3::new(1.0, 0.0, 0.0).normalize() * Config::INSECT_SPEED),
        BrownianMotion {
            wander_strength: Config::INSECT_WANDER_STRENGTH,
        },
    )).id();

    app.update();

    let vel = app.world().get::<Velocity>(entity).unwrap().0;
    let speed = vel.length();
    assert!(
        (speed - Config::INSECT_SPEED).abs() < 1.0,
        "Brownian motion should maintain roughly constant speed. Expected ~{}, got {}",
        Config::INSECT_SPEED,
        speed
    );
}

#[test]
fn brownian_motion_does_not_affect_birds() {
    let mut app = test_app();
    app.add_systems(Update, brownian_motion_system);

    let initial_vel = Vec3::new(1.0, 0.0, 0.0) * Config::BIRD_SPEED;
    let entity = app.world_mut().spawn((
        Bird,
        Velocity(initial_vel),
    )).id();

    app.update();

    let vel = app.world().get::<Velocity>(entity).unwrap().0;
    assert_eq!(
        vel, initial_vel,
        "Brownian motion should not affect Bird entities"
    );
}

// =============================================================
// Spatial index rebuild system tests
// =============================================================

#[test]
fn rebuild_spatial_index_populates_from_entities() {
    let mut app = test_app();
    app.add_systems(Update, rebuild_spatial_index);

    app.world_mut().spawn((
        Insect,
        Transform::from_translation(Vec3::new(10.0, 10.0, 10.0)),
    ));
    app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::new(20.0, 20.0, 20.0)),
    ));

    app.update();

    let spatial = app.world().resource::<SpatialIndex>();
    let nearby_insect = spatial.get_nearby(Vec3::new(10.0, 10.0, 10.0));
    let nearby_bird = spatial.get_nearby(Vec3::new(20.0, 20.0, 20.0));
    assert!(
        !nearby_insect.is_empty(),
        "Spatial index should contain insect after rebuild"
    );
    assert!(
        !nearby_bird.is_empty(),
        "Spatial index should contain bird after rebuild"
    );
}

// =============================================================
// Flocking system tests
// =============================================================

#[test]
fn flocking_cohesion_steers_bird_toward_group() {
    let mut app = test_app();
    app.add_systems(Update, (rebuild_spatial_index, flocking_system).chain());

    let flocking = Flocking {
        separation_weight: 0.0, // disable separation for this test
        alignment_weight: 0.0,  // disable alignment for this test
        cohesion_weight: 5.0,   // strong cohesion
    };

    // Lone bird far from group, moving away
    let lone_bird = app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::new(30.0, 0.0, 0.0)),
        Velocity(Vec3::new(1.0, 0.0, 0.0).normalize() * Config::BIRD_SPEED),
        flocking.clone(),
    )).id();

    // Group of birds at origin
    for i in 0..5 {
        app.world_mut().spawn((
            Bird,
            Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)),
            Velocity(Vec3::new(0.0, 1.0, 0.0).normalize() * Config::BIRD_SPEED),
            flocking.clone(),
        ));
    }

    app.update();

    let vel = app.world().get::<Velocity>(lone_bird).unwrap().0;
    // Cohesion should steer the lone bird back toward the group (negative X direction)
    assert!(
        vel.x < Config::BIRD_SPEED,
        "Cohesion should reduce the lone bird's +X velocity (steering toward group). Got vx={}",
        vel.x
    );
}

#[test]
fn flocking_separation_pushes_bird_away_from_close_neighbor() {
    let mut app = test_app();
    app.add_systems(Update, (rebuild_spatial_index, flocking_system).chain());

    let flocking = Flocking {
        separation_weight: 5.0, // strong separation
        alignment_weight: 0.0,
        cohesion_weight: 0.0,
    };

    // Two birds very close together, both heading +X
    let bird_a = app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Velocity(Vec3::X * Config::BIRD_SPEED),
        flocking.clone(),
    )).id();

    app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        Velocity(Vec3::X * Config::BIRD_SPEED),
        flocking.clone(),
    ));

    app.update();

    let vel = app.world().get::<Velocity>(bird_a).unwrap().0;
    // Separation from bird at +1x should push bird_a toward -X
    assert!(
        vel.x < Config::BIRD_SPEED,
        "Separation should steer bird away from close neighbor. Got vx={}",
        vel.x
    );
}

#[test]
fn flocking_does_not_affect_insects() {
    let mut app = test_app();
    app.add_systems(Update, (rebuild_spatial_index, flocking_system).chain());

    let initial_vel = Vec3::Y * Config::INSECT_SPEED;
    let insect = app.world_mut().spawn((
        Insect,
        Transform::from_translation(Vec3::ZERO),
        Velocity(initial_vel),
    )).id();

    app.update();

    let vel = app.world().get::<Velocity>(insect).unwrap().0;
    assert_eq!(vel, initial_vel, "Flocking should not affect insects");
}

// =============================================================
// Predator sight system tests
// =============================================================

#[test]
fn predator_steers_toward_insect_in_cone() {
    let mut app = test_app();
    app.add_systems(Update, (rebuild_spatial_index, predator_sight_system).chain());

    // Bird at origin, facing +X
    let bird = app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::ZERO),
        Velocity(Vec3::X * Config::BIRD_SPEED),
        Predator {
            sight_range: Config::BIRD_SIGHT_RANGE,
            sight_half_angle: Config::BIRD_SIGHT_HALF_ANGLE,
        },
    )).id();

    // Insect ahead and slightly to the +Y side — within the forward cone
    app.world_mut().spawn((
        Insect,
        Transform::from_translation(Vec3::new(20.0, 5.0, 0.0)),
    ));

    app.update();

    let vel = app.world().get::<Velocity>(bird).unwrap().0;
    // Bird should steer slightly toward +Y to chase the insect
    assert!(
        vel.y > 0.0,
        "Predator should steer toward insect in cone. Got vy={}",
        vel.y
    );
}

#[test]
fn predator_ignores_insect_behind_it() {
    let mut app = test_app();
    app.add_systems(Update, (rebuild_spatial_index, predator_sight_system).chain());

    // Bird at origin, facing +X
    let initial_vel = Vec3::X * Config::BIRD_SPEED;
    let bird = app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::ZERO),
        Velocity(initial_vel),
        Predator {
            sight_range: Config::BIRD_SIGHT_RANGE,
            sight_half_angle: Config::BIRD_SIGHT_HALF_ANGLE,
        },
    )).id();

    // Insect directly behind the bird (negative X)
    app.world_mut().spawn((
        Insect,
        Transform::from_translation(Vec3::new(-20.0, 0.0, 0.0)),
    ));

    app.update();

    let vel = app.world().get::<Velocity>(bird).unwrap().0;
    // Bird should not change direction for an insect behind it
    assert_eq!(
        vel, initial_vel,
        "Predator should ignore insect behind it"
    );
}

#[test]
fn predator_ignores_insect_outside_range() {
    let mut app = test_app();
    app.add_systems(Update, (rebuild_spatial_index, predator_sight_system).chain());

    let initial_vel = Vec3::X * Config::BIRD_SPEED;
    let bird = app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::ZERO),
        Velocity(initial_vel),
        Predator {
            sight_range: Config::BIRD_SIGHT_RANGE,
            sight_half_angle: Config::BIRD_SIGHT_HALF_ANGLE,
        },
    )).id();

    // Insect far ahead, outside sight range
    app.world_mut().spawn((
        Insect,
        Transform::from_translation(Vec3::new(500.0, 0.0, 0.0)),
    ));

    app.update();

    let vel = app.world().get::<Velocity>(bird).unwrap().0;
    assert_eq!(
        vel, initial_vel,
        "Predator should ignore insect outside sight range"
    );
}

// =============================================================
// Eating system tests
// =============================================================

#[test]
fn eating_system_despawns_insect_when_bird_is_close() {
    let mut app = test_app();
    app.add_systems(Update, (rebuild_spatial_index, eating_system).chain());

    // Bird and insect at same position
    app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::ZERO),
        Velocity(Vec3::X * Config::BIRD_SPEED),
    ));

    let insect = app.world_mut().spawn((
        Insect,
        Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)), // within eating distance
    )).id();

    app.update();

    // The insect should be despawned (or at least the entity should be gone)
    assert!(
        app.world().get_entity(insect).is_err(),
        "Insect within eating distance should be despawned"
    );
}

#[test]
fn eating_system_does_not_despawn_distant_insect() {
    let mut app = test_app();
    app.add_systems(Update, (rebuild_spatial_index, eating_system).chain());

    app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::ZERO),
        Velocity(Vec3::X * Config::BIRD_SPEED),
    ));

    let insect = app.world_mut().spawn((
        Insect,
        Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)), // far away
    )).id();

    app.update();

    assert!(
        app.world().get_entity(insect).is_ok(),
        "Insect far from bird should NOT be despawned"
    );
}

/// Full pipeline: bird eats insect → InsectEaten message triggers lifecycle transition.
#[test]
fn eating_triggers_nesting_lifecycle() {
    use vivarium::systems::nesting::bird_lifecycle_system;
    use vivarium::nav_graph::{NavGraph, NavNodeKind};

    let mut app = test_app();

    // Nav graph with a branch node
    let mut nav = NavGraph::new();
    nav.add_node(Vec3::new(0.0, -200.0, 0.0), NavNodeKind::Ground);
    nav.add_node(Vec3::new(0.0, -200.0, 0.0), NavNodeKind::TreeBase);
    nav.add_edge(0, 1);
    nav.add_node(Vec3::new(0.0, -190.0, 0.0), NavNodeKind::Branch);
    nav.add_edge(1, 2);
    app.insert_resource(nav);

    // eating_system emits InsectEaten, bird_lifecycle_system reads it
    app.add_systems(Update, (
        rebuild_spatial_index,
        eating_system,
        bird_lifecycle_system,
    ).chain());

    // Bird in Hunting phase, right next to an insect
    let bird = app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::ZERO),
        Velocity(Vec3::X * Config::BIRD_SPEED),
        BirdNestingState::default(), // Hunting
    )).id();

    app.world_mut().spawn((
        Insect,
        Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
    ));

    app.update();

    let nesting = app.world().get::<BirdNestingState>(bird).unwrap();
    assert_eq!(
        nesting.phase, BirdLifecycle::FlyingToNest,
        "Eating an insect should trigger Hunting → FlyingToNest via InsectEaten message"
    );
}

#[test]
fn eating_system_bird_does_not_eat_other_birds() {
    let mut app = test_app();
    app.add_systems(Update, (rebuild_spatial_index, eating_system).chain());

    let bird_a = app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::ZERO),
        Velocity(Vec3::X * Config::BIRD_SPEED),
    )).id();

    app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        Velocity(Vec3::X * Config::BIRD_SPEED),
    ));

    app.update();

    assert!(
        app.world().get_entity(bird_a).is_ok(),
        "Birds should not eat other birds"
    );
}
