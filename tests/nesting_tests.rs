use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use vivarium::components::*;
use vivarium::config::Config;
use vivarium::nav_graph::{NavGraph, NavNodeKind};
use vivarium::spatial::SpatialIndex;
use vivarium::systems::nesting::{bird_lifecycle_system, bird_fly_to_target_system, hatchling_alert_system};
use vivarium::systems::spatial_update::rebuild_spatial_index;
use vivarium::wind::Wind;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f64(1.0 / 60.0),
    ));
    app.insert_resource(SpatialIndex::new(Config::SPATIAL_CELL_SIZE));
    app.insert_resource(Wind::default());
    app.add_message::<InsectEaten>();
    app
}

fn test_nav_graph() -> NavGraph {
    let mut nav = NavGraph::new();
    // Ground node at origin
    nav.add_node(Vec3::new(0.0, -200.0, 0.0), NavNodeKind::Ground);
    // Tree base
    nav.add_node(Vec3::new(0.0, -200.0, 0.0), NavNodeKind::TreeBase);
    nav.add_edge(0, 1);
    // Branch node at height (simulates a branch tip)
    nav.add_node(Vec3::new(0.0, -190.0, 0.0), NavNodeKind::Branch);
    nav.add_edge(1, 2);
    // Another branch node (unoccupied)
    nav.add_node(Vec3::new(10.0, -185.0, 0.0), NavNodeKind::Branch);
    nav.add_edge(2, 3);
    nav
}

/// After eating an insect, bird should transition from Hunting to FlyingToNest.
#[test]
fn bird_transitions_to_nesting_after_eating() {
    let mut app = test_app();
    app.insert_resource(test_nav_graph());
    app.add_systems(Update, bird_lifecycle_system);

    // Spawn a bird in Hunting phase
    let bird = app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Velocity(Vec3::X * Config::BIRD_SPEED),
        BirdNestingState::default(),
    )).id();

    // Manually send an InsectEaten message
    app.world_mut().write_message(InsectEaten { bird });

    app.update();

    let nesting = app.world().get::<BirdNestingState>(bird).unwrap();
    assert_eq!(
        nesting.phase, BirdLifecycle::FlyingToNest,
        "Bird should transition to FlyingToNest after eating an insect"
    );
    assert!(nesting.nest_nav_node.is_some(), "Should have picked a branch node");
}

/// Bird in Building phase should spawn a Nest entity when timer expires.
#[test]
fn bird_builds_nest_at_branch() {
    let mut app = test_app();
    app.insert_resource(test_nav_graph());
    app.add_systems(Update, bird_lifecycle_system);

    // Spawn bird already in Building phase
    app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::new(0.0, -190.0, 0.0)),
        Velocity(Vec3::ZERO),
        BirdNestingState {
            phase: BirdLifecycle::Building,
            nest: None,
            nest_nav_node: Some(2),
            timer: 0.02, // will expire in 2 frames at 1/60
            insects_eaten: 0,
        },
    ));

    // Run enough frames for the timer to expire + one more for Commands to apply
    app.update();
    app.update();
    app.update(); // Commands from previous frame apply here

    // Check that a Nest entity was spawned
    let nest_count = app.world_mut().query::<&Nest>().iter(app.world()).count();
    assert_eq!(nest_count, 1, "A nest should have been spawned");
}

/// Bird in Incubating phase should spawn a Hatchling when timer expires.
#[test]
fn bird_incubates_and_hatches() {
    let mut app = test_app();
    app.insert_resource(test_nav_graph());
    app.add_systems(Update, bird_lifecycle_system);

    // First spawn a nest
    let bird_entity = Entity::from_bits(999); // placeholder
    let nest = app.world_mut().spawn((
        Nest {
            parent_bird: bird_entity,
            nav_node: 2,
            has_egg: true,
            has_hatchling: false,
        },
        Transform::from_translation(Vec3::new(0.0, -190.0, 0.0)),
    )).id();

    // Spawn bird in Incubating phase with tiny timer
    let bird = app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::new(0.0, -190.0, 0.0)),
        Velocity(Vec3::ZERO),
        BirdNestingState {
            phase: BirdLifecycle::Incubating,
            nest: Some(nest),
            nest_nav_node: Some(2),
            timer: 0.02,
            insects_eaten: 0,
        },
    )).id();

    // Fix the nest's parent_bird to the actual entity
    app.world_mut().entity_mut(nest).insert(Nest {
        parent_bird: bird,
        nav_node: 2,
        has_egg: true,
        has_hatchling: false,
    });

    app.update();
    app.update();
    app.update(); // Commands from previous frame apply here

    let hatchling_count = app.world_mut().query::<&Hatchling>().iter(app.world()).count();
    assert_eq!(hatchling_count, 1, "A hatchling should have been spawned");

    let nesting = app.world().get::<BirdNestingState>(bird).unwrap();
    assert_eq!(nesting.phase, BirdLifecycle::Parenting, "Bird should be in Parenting phase");
}

/// Parenting bird delivers food every other insect.
#[test]
fn bird_delivers_food_alternating() {
    let mut app = test_app();
    app.insert_resource(test_nav_graph());
    app.add_systems(Update, bird_lifecycle_system);

    let nest = app.world_mut().spawn((
        Nest {
            parent_bird: Entity::from_bits(999),
            nav_node: 2,
            has_egg: false,
            has_hatchling: true,
        },
        Transform::from_translation(Vec3::new(0.0, -190.0, 0.0)),
    )).id();

    let bird = app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
        Velocity(Vec3::X * Config::BIRD_SPEED),
        BirdNestingState {
            phase: BirdLifecycle::Parenting,
            nest: Some(nest),
            nest_nav_node: Some(2),
            timer: 0.0,
            insects_eaten: 0,
        },
    )).id();

    // First insect: insects_eaten becomes 1 (odd), stay Parenting
    app.world_mut().write_message(InsectEaten { bird });
    app.update();

    let nesting = app.world().get::<BirdNestingState>(bird).unwrap();
    assert_eq!(nesting.phase, BirdLifecycle::Parenting, "First insect: should stay Parenting");

    // Second insect: insects_eaten becomes 2 (even), fly to nest to deliver
    app.world_mut().write_message(InsectEaten { bird });
    app.update();

    let nesting = app.world().get::<BirdNestingState>(bird).unwrap();
    assert_eq!(nesting.phase, BirdLifecycle::FlyingToNest, "Second insect: should fly to nest to deliver food");
}

/// Bird in FlyingToNest should steer toward nest position.
#[test]
fn bird_steers_toward_nest() {
    let mut app = test_app();
    app.insert_resource(test_nav_graph());
    app.add_systems(Update, bird_fly_to_target_system);

    let bird = app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
        Velocity(Vec3::X * Config::BIRD_SPEED),
        BirdNestingState {
            phase: BirdLifecycle::FlyingToNest,
            nest: None,
            nest_nav_node: Some(2), // branch node at (0, -190, 0)
            timer: 0.0,
            insects_eaten: 0,
        },
    )).id();

    app.update();

    let vel = app.world().get::<Velocity>(bird).unwrap();
    // Nest is at (0, -190, 0), bird at (100, 0, 0)
    // Velocity should point roughly toward negative X and negative Y
    assert!(vel.0.x < 0.0, "Should steer toward nest (negative X direction)");
    assert!(vel.0.y < 0.0, "Should steer toward nest (negative Y direction)");
}

/// Hunt system should skip birds that are in Building phase.
#[test]
fn hunt_system_skips_nesting_birds() {
    let mut app = test_app();
    app.insert_resource(test_nav_graph());
    app.add_systems(Update, (rebuild_spatial_index, vivarium::systems::hunt::hunt_system).chain());

    let bird = app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Velocity(Vec3::X * Config::BIRD_SPEED),
        Predator {
            sight_range: Config::BIRD_SIGHT_RANGE,
            sight_half_angle: Config::BIRD_SIGHT_HALF_ANGLE,
        },
        HuntState::default(),
        Wander { strength: Config::BIRD_WANDER_STRENGTH },
        BirdNestingState {
            phase: BirdLifecycle::Building,
            nest: None,
            nest_nav_node: Some(2),
            timer: 3.0,
            insects_eaten: 0,
        },
    )).id();

    // Place an insect right in front of the bird
    app.world_mut().spawn((
        Insect,
        Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        Velocity(Vec3::ZERO),
    ));

    app.update();

    // Hunt system should not have modified the hunt state
    let hunt_state = app.world().get::<HuntState>(bird).unwrap();
    assert_eq!(hunt_state.phase, HuntPhase::Searching, "Should still be Searching (hunt skipped)");
}

/// Hatchling should alert when squirrel is close, triggering parent defense.
#[test]
fn hatchling_alerts_and_parent_defends() {
    let mut app = test_app();
    app.insert_resource(test_nav_graph());
    app.add_systems(Update, hatchling_alert_system);

    let bird = app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
        Velocity(Vec3::X * Config::BIRD_SPEED),
        BirdNestingState {
            phase: BirdLifecycle::Parenting,
            nest: None,
            nest_nav_node: Some(2),
            timer: 0.0,
            insects_eaten: 0,
        },
    )).id();

    let nest = app.world_mut().spawn((
        Nest {
            parent_bird: bird,
            nav_node: 2,
            has_egg: false,
            has_hatchling: true,
        },
        Transform::from_translation(Vec3::new(0.0, -190.0, 0.0)),
    )).id();

    // Hatchling at nest position
    app.world_mut().spawn((
        Hatchling {
            nest,
            parent_bird: bird,
            alert: false,
        },
        Transform::from_translation(Vec3::new(0.0, -190.0, 0.0)),
    ));

    // Squirrel very close to hatchling, with SquirrelTarget
    app.world_mut().spawn((
        Squirrel,
        SquirrelState::default(),
        SquirrelTarget {
            hatchling: Entity::from_bits(999),
            nest_nav_node: 2,
        },
        Transform::from_translation(Vec3::new(5.0, -190.0, 0.0)),
    ));

    app.update();

    // Hatchling should be alerting
    let hatchling = app.world_mut().query::<&Hatchling>().iter(app.world()).next().unwrap();
    assert!(hatchling.alert, "Hatchling should be alerting");

    // Parent should be defending
    let nesting = app.world().get::<BirdNestingState>(bird).unwrap();
    assert_eq!(nesting.phase, BirdLifecycle::Defending, "Parent should switch to Defending");
}
