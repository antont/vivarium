use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use vivarium::components::*;
use vivarium::config::Config;
use vivarium::spatial::SpatialIndex;
use vivarium::systems::hunt::hunt_system;
use vivarium::systems::spatial_update::rebuild_spatial_index;
use vivarium::systems::swarm_cohesion::swarm_cohesion_system;
use vivarium::wind::Wind;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f64(1.0 / 60.0),
    ));
    app.insert_resource(SpatialIndex::new(Config::SPATIAL_CELL_SIZE));
    app.insert_resource(Wind::default());
    app
}

// =============================================================
// Insect swarm cohesion tests
// =============================================================

#[test]
fn swarm_cohesion_steers_lone_insect_toward_group() {
    let mut app = test_app();
    app.add_systems(Update, (rebuild_spatial_index, swarm_cohesion_system).chain());

    let cohesion = SwarmCohesion {
        radius: Config::SWARM_COHESION_RADIUS,
        weight: Config::SWARM_COHESION_WEIGHT,
    };

    // Lone insect heading +X, group is to the +Y side
    let lone = app.world_mut().spawn((
        Insect,
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Velocity(Vec3::X * Config::INSECT_SPEED),
        cohesion.clone(),
    )).id();

    // Cluster of insects offset in +Y
    for i in 0..5 {
        app.world_mut().spawn((
            Insect,
            Transform::from_translation(Vec3::new(0.0, 10.0 + i as f32, 0.0)),
            Velocity(Vec3::Y * Config::INSECT_SPEED),
            cohesion.clone(),
        ));
    }

    app.update();

    let vel = app.world().get::<Velocity>(lone).unwrap().0;
    // Should steer toward +Y group — vy should become positive
    assert!(
        vel.y > 0.0,
        "Cohesion should steer lone insect toward group. Got vy={}",
        vel.y
    );
}

#[test]
fn swarm_cohesion_does_not_affect_distant_insects() {
    let mut app = test_app();
    app.add_systems(Update, (rebuild_spatial_index, swarm_cohesion_system).chain());

    let cohesion = SwarmCohesion {
        radius: Config::SWARM_COHESION_RADIUS,
        weight: Config::SWARM_COHESION_WEIGHT,
    };

    let initial_vel = Vec3::X * Config::INSECT_SPEED;
    let far_insect = app.world_mut().spawn((
        Insect,
        Transform::from_translation(Vec3::new(500.0, 0.0, 0.0)),
        Velocity(initial_vel),
        cohesion.clone(),
    )).id();

    app.world_mut().spawn((
        Insect,
        Transform::from_translation(Vec3::ZERO),
        Velocity(Vec3::Y * Config::INSECT_SPEED),
        cohesion.clone(),
    ));

    app.update();

    let vel = app.world().get::<Velocity>(far_insect).unwrap().0;
    assert_eq!(vel, initial_vel, "Distant insects should not be affected by cohesion");
}

// =============================================================
// Bird hunting state tests
// =============================================================

#[test]
fn bird_starts_in_searching_state() {
    let state = HuntState::default();
    assert_eq!(state.phase, HuntPhase::Searching);
}

#[test]
fn bird_transitions_to_circling_when_insects_spotted() {
    let mut app = test_app();
    app.add_systems(Update, (rebuild_spatial_index, hunt_system).chain());

    let bird = app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::ZERO),
        Velocity(Vec3::X * Config::BIRD_SPEED),
        Predator {
            sight_range: Config::BIRD_SIGHT_RANGE,
            sight_half_angle: Config::BIRD_SIGHT_HALF_ANGLE,
        },
        HuntState::default(),
        Wander { strength: Config::BIRD_WANDER_STRENGTH },
    )).id();

    // Insect in front of bird, within sight cone
    app.world_mut().spawn((
        Insect,
        Transform::from_translation(Vec3::new(30.0, 0.0, 0.0)),
    ));

    app.update();

    let state = app.world().get::<HuntState>(bird).unwrap();
    assert_eq!(
        state.phase,
        HuntPhase::Circling,
        "Bird should transition to Circling when insects spotted"
    );
}

#[test]
fn bird_transitions_to_diving_after_circling() {
    let mut app = test_app();
    app.add_systems(Update, (rebuild_spatial_index, hunt_system).chain());

    let bird = app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::ZERO),
        Velocity(Vec3::X * Config::BIRD_SPEED),
        Predator {
            sight_range: Config::BIRD_SIGHT_RANGE,
            sight_half_angle: Config::BIRD_SIGHT_HALF_ANGLE,
        },
        HuntState {
            phase: HuntPhase::Circling,
            timer: Config::HUNT_CIRCLE_DURATION + 0.1, // timer expired
            target_pos: Vec3::new(30.0, 0.0, 0.0),
        },
        Wander { strength: Config::BIRD_WANDER_STRENGTH },
    )).id();

    // Insect at target position
    app.world_mut().spawn((
        Insect,
        Transform::from_translation(Vec3::new(30.0, 0.0, 0.0)),
    ));

    app.update();

    let state = app.world().get::<HuntState>(bird).unwrap();
    assert_eq!(
        state.phase,
        HuntPhase::Diving,
        "Bird should transition to Diving after circle timer expires"
    );
}

#[test]
fn bird_dives_faster_than_normal_speed() {
    let mut app = test_app();
    app.add_systems(Update, (rebuild_spatial_index, hunt_system).chain());

    let bird = app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::ZERO),
        Velocity(Vec3::X * Config::BIRD_SPEED),
        Predator {
            sight_range: Config::BIRD_SIGHT_RANGE,
            sight_half_angle: Config::BIRD_SIGHT_HALF_ANGLE,
        },
        HuntState {
            phase: HuntPhase::Diving,
            timer: 0.0,
            target_pos: Vec3::new(30.0, 0.0, 0.0),
        },
        Wander { strength: Config::BIRD_WANDER_STRENGTH },
    )).id();

    // Insect at target
    app.world_mut().spawn((
        Insect,
        Transform::from_translation(Vec3::new(30.0, 0.0, 0.0)),
    ));

    app.update();

    let speed = app.world().get::<Velocity>(bird).unwrap().0.length();
    assert!(
        speed > Config::BIRD_SPEED,
        "Diving bird should be faster than normal. Got speed={}, expected > {}",
        speed,
        Config::BIRD_SPEED
    );
}

// =============================================================
// Bird wandering tests
// =============================================================

#[test]
fn bird_wanders_when_no_insects_visible() {
    let mut app = test_app();
    app.add_systems(Update, (rebuild_spatial_index, hunt_system).chain());

    let bird = app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::ZERO),
        Velocity(Vec3::X * Config::BIRD_SPEED),
        Predator {
            sight_range: Config::BIRD_SIGHT_RANGE,
            sight_half_angle: Config::BIRD_SIGHT_HALF_ANGLE,
        },
        HuntState::default(),
        Wander { strength: Config::BIRD_WANDER_STRENGTH },
    )).id();

    // No insects in the world — bird should wander
    for _ in 0..20 {
        app.update();
    }

    let vel = app.world().get::<Velocity>(bird).unwrap().0;
    let angle = vel.normalize().dot(Vec3::X).acos();
    assert!(
        angle > 0.01,
        "Bird should wander (change direction) when no insects visible. Angle from +X: {}",
        angle
    );
}

#[test]
fn bird_stays_searching_when_no_insects() {
    let mut app = test_app();
    app.add_systems(Update, (rebuild_spatial_index, hunt_system).chain());

    let bird = app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::ZERO),
        Velocity(Vec3::X * Config::BIRD_SPEED),
        Predator {
            sight_range: Config::BIRD_SIGHT_RANGE,
            sight_half_angle: Config::BIRD_SIGHT_HALF_ANGLE,
        },
        HuntState::default(),
        Wander { strength: Config::BIRD_WANDER_STRENGTH },
    )).id();

    app.update();

    let state = app.world().get::<HuntState>(bird).unwrap();
    assert_eq!(
        state.phase,
        HuntPhase::Searching,
        "Bird should remain Searching when no insects visible"
    );
}
