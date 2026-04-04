use bevy::prelude::*;
use ecology::components::*;
use ecology::config::Config;
use ecology::spatial::SpatialIndex;

// We test the spawning functions directly, not the full App with rendering.
// The spawn functions will be public in lib.rs for testability.
use ecology::spawn_insects;
use ecology::spawn_birds;

// =============================================================
// Insect spawning tests
// =============================================================

#[test]
fn spawn_insects_creates_correct_count() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let count = spawn_insects(app.world_mut());
    assert_eq!(
        count,
        Config::INSECT_COUNT,
        "Should spawn exactly INSECT_COUNT insects"
    );
}

#[test]
fn spawned_insects_have_required_components() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    spawn_insects(app.world_mut());

    let mut query = app.world_mut().query_filtered::<(
        &Insect,
        &Velocity,
        &BrownianMotion,
        &BoundaryWrap,
        &Transform,
    ), With<Insect>>();

    let count = query.iter(app.world()).count();
    assert_eq!(
        count,
        Config::INSECT_COUNT,
        "All insects should have Insect, Velocity, BrownianMotion, BoundaryWrap, Transform"
    );
}

#[test]
fn spawned_insects_are_within_world_bounds() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    spawn_insects(app.world_mut());

    let half = Config::WORLD_HALF_SIZE;
    let mut query = app.world_mut().query_filtered::<&Transform, With<Insect>>();

    for transform in query.iter(app.world()) {
        let pos = transform.translation;
        assert!(
            pos.x.abs() <= half && pos.y.abs() <= half && pos.z.abs() <= half,
            "Insect spawned outside world bounds: {:?}",
            pos
        );
    }
}

#[test]
fn spawned_insects_have_correct_speed() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    spawn_insects(app.world_mut());

    let mut query = app.world_mut().query_filtered::<&Velocity, With<Insect>>();

    for velocity in query.iter(app.world()) {
        let speed = velocity.0.length();
        assert!(
            (speed - Config::INSECT_SPEED).abs() < 0.1,
            "Insect speed should be ~{}, got {}",
            Config::INSECT_SPEED,
            speed
        );
    }
}

// =============================================================
// Bird spawning tests
// =============================================================

#[test]
fn spawn_birds_creates_correct_count() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let count = spawn_birds(app.world_mut());
    assert_eq!(
        count,
        Config::BIRD_COUNT,
        "Should spawn exactly BIRD_COUNT birds"
    );
}

#[test]
fn spawned_birds_have_required_components() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    spawn_birds(app.world_mut());

    let mut query = app.world_mut().query_filtered::<(
        &Bird,
        &Velocity,
        &Predator,
        &Flocking,
        &BoundaryWrap,
        &Transform,
    ), With<Bird>>();

    let count = query.iter(app.world()).count();
    assert_eq!(
        count,
        Config::BIRD_COUNT,
        "All birds should have Bird, Velocity, Predator, Flocking, BoundaryWrap, Transform"
    );
}

#[test]
fn spawned_birds_are_within_world_bounds() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    spawn_birds(app.world_mut());

    let half = Config::WORLD_HALF_SIZE;
    let mut query = app.world_mut().query_filtered::<&Transform, With<Bird>>();

    for transform in query.iter(app.world()) {
        let pos = transform.translation;
        assert!(
            pos.x.abs() <= half && pos.y.abs() <= half && pos.z.abs() <= half,
            "Bird spawned outside world bounds: {:?}",
            pos
        );
    }
}

#[test]
fn spawned_birds_have_correct_speed() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    spawn_birds(app.world_mut());

    let mut query = app.world_mut().query_filtered::<&Velocity, With<Bird>>();

    for velocity in query.iter(app.world()) {
        let speed = velocity.0.length();
        assert!(
            (speed - Config::BIRD_SPEED).abs() < 0.1,
            "Bird speed should be ~{}, got {}",
            Config::BIRD_SPEED,
            speed
        );
    }
}

#[test]
fn spawned_birds_have_correct_sight_params() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    spawn_birds(app.world_mut());

    let mut query = app.world_mut().query_filtered::<&Predator, With<Bird>>();

    for predator in query.iter(app.world()) {
        assert_eq!(predator.sight_range, Config::BIRD_SIGHT_RANGE);
        assert_eq!(predator.sight_half_angle, Config::BIRD_SIGHT_HALF_ANGLE);
    }
}

#[test]
fn no_entity_is_both_insect_and_bird() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    spawn_insects(app.world_mut());
    spawn_birds(app.world_mut());

    let mut query = app.world_mut().query_filtered::<Entity, (With<Insect>, With<Bird>)>();
    let count = query.iter(app.world()).count();
    assert_eq!(count, 0, "No entity should be both Insect and Bird");
}
