use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use vivarium::components::*;
use vivarium::config::Config;
use vivarium::nav_graph::{NavGraph, NavNodeKind};
use vivarium::spatial::SpatialIndex;
use vivarium::squirrel::{squirrel_behavior_system, squirrel_hatchling_detection_system, squirrel_flee_system};
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

fn test_nav_graph() -> NavGraph {
    let mut nav = NavGraph::new();
    // Ground nodes
    nav.add_node(Vec3::new(0.0, -200.0, 0.0), NavNodeKind::Ground);
    nav.add_node(Vec3::new(40.0, -200.0, 0.0), NavNodeKind::Ground);
    nav.add_node(Vec3::new(80.0, -200.0, 0.0), NavNodeKind::Ground);
    nav.add_edge(0, 1);
    nav.add_edge(1, 2);
    // Tree base
    nav.add_node(Vec3::new(0.0, -200.0, 0.0), NavNodeKind::TreeBase);
    nav.add_edge(0, 3);
    // Branch node (where the nest/hatchling is)
    nav.add_node(Vec3::new(0.0, -190.0, 0.0), NavNodeKind::Branch);
    nav.add_edge(3, 4);
    nav
}

/// Idle squirrel near a hatchling should detect it and start hunting.
#[test]
fn squirrel_detects_nearby_hatchling() {
    let mut app = test_app();
    app.insert_resource(test_nav_graph());
    app.add_systems(Update, squirrel_hatchling_detection_system);

    // Hatchling at branch node position
    let nest = app.world_mut().spawn((
        Nest {
            parent_bird: Entity::from_bits(999),
            nav_node: 4,
            has_egg: false,
            has_hatchling: true,
        },
        Transform::from_translation(Vec3::new(0.0, -190.0, 0.0)),
    )).id();

    let hatchling = app.world_mut().spawn((
        Hatchling {
            nest,
            parent_bird: Entity::from_bits(999),
            alert: false,
        },
        Transform::from_translation(Vec3::new(0.0, -190.0, 0.0)),
    )).id();

    // Squirrel within sight range (30 units away)
    let squirrel = app.world_mut().spawn((
        Squirrel,
        SquirrelState::default(), // Idle phase
        Transform::from_translation(Vec3::new(30.0, -200.0, 0.0)),
    )).id();

    app.update();

    let state = app.world().get::<SquirrelState>(squirrel).unwrap();
    assert_eq!(state.phase, SquirrelPhase::Hunting, "Squirrel should start hunting");
    assert!(!state.path.is_empty(), "Squirrel should have a path to the hatchling");

    // Should have SquirrelTarget component
    assert!(
        app.world().get::<SquirrelTarget>(squirrel).is_some(),
        "Squirrel should have SquirrelTarget component"
    );
}

/// Squirrel far from hatchling should stay idle.
#[test]
fn squirrel_ignores_distant_hatchling() {
    let mut app = test_app();
    app.insert_resource(test_nav_graph());
    app.add_systems(Update, squirrel_hatchling_detection_system);

    let nest = app.world_mut().spawn((
        Nest {
            parent_bird: Entity::from_bits(999),
            nav_node: 4,
            has_egg: false,
            has_hatchling: true,
        },
        Transform::from_translation(Vec3::new(0.0, -190.0, 0.0)),
    )).id();

    app.world_mut().spawn((
        Hatchling {
            nest,
            parent_bird: Entity::from_bits(999),
            alert: false,
        },
        Transform::from_translation(Vec3::new(0.0, -190.0, 0.0)),
    ));

    // Squirrel far away (beyond sight range)
    let squirrel = app.world_mut().spawn((
        Squirrel,
        SquirrelState::default(),
        Transform::from_translation(Vec3::new(200.0, -200.0, 0.0)),
    )).id();

    app.update();

    let state = app.world().get::<SquirrelState>(squirrel).unwrap();
    assert_eq!(state.phase, SquirrelPhase::Idle, "Squirrel should stay idle when hatchling is far");
}

/// When parent bird is near nest, hunting squirrel should flee.
#[test]
fn squirrel_flees_when_parent_arrives() {
    let mut app = test_app();
    app.insert_resource(test_nav_graph());
    app.add_systems(Update, squirrel_flee_system);

    // Parent bird near the nest
    app.world_mut().spawn((
        Bird,
        Transform::from_translation(Vec3::new(2.0, -190.0, 0.0)),
        Velocity(Vec3::ZERO),
        BirdNestingState {
            phase: BirdLifecycle::Defending,
            nest: None,
            nest_nav_node: Some(4),
            timer: 0.0,
            insects_eaten: 0,
        },
    ));

    // Squirrel hunting near the nest
    let squirrel = app.world_mut().spawn((
        Squirrel,
        SquirrelState {
            phase: SquirrelPhase::Hunting,
            path: vec![3, 4],
            path_index: 1,
            progress: 0.9,
            timer: 0.0,
            last_normal: Vec3::Y,
        },
        SquirrelTarget {
            hatchling: Entity::from_bits(999),
            nest_nav_node: 4,
        },
        Transform::from_translation(Vec3::new(3.0, -191.0, 0.0)),
    )).id();

    app.update();

    let state = app.world().get::<SquirrelState>(squirrel).unwrap();
    assert_eq!(state.phase, SquirrelPhase::Fleeing, "Squirrel should flee when parent is near");
    assert!(
        app.world().get::<SquirrelTarget>(squirrel).is_none(),
        "SquirrelTarget should be removed when fleeing"
    );
}

/// Fleeing squirrel should return to idle when path is done.
#[test]
fn squirrel_returns_to_idle_after_fleeing() {
    let mut app = test_app();
    app.insert_resource(test_nav_graph());
    app.add_systems(Update, squirrel_behavior_system);

    let squirrel = app.world_mut().spawn((
        Squirrel,
        SquirrelState {
            phase: SquirrelPhase::Fleeing,
            path: vec![0, 1],
            path_index: 1, // at end of path
            progress: 0.0,
            timer: 0.0,
            last_normal: Vec3::Y,
        },
        Transform::from_translation(Vec3::new(40.0, -200.0, 0.0)),
    )).id();

    app.update();

    let state = app.world().get::<SquirrelState>(squirrel).unwrap();
    assert_eq!(state.phase, SquirrelPhase::Idle, "Squirrel should return to idle after fleeing");
}
