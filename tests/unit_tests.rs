use bevy::prelude::*;
use vivarium::components::*;
use vivarium::config::{Colors, Config};
use vivarium::spatial::SpatialIndex;

// =============================================================
// Config tests — verify simulation constants are sensible
// =============================================================

#[test]
fn config_world_half_size_is_positive() {
    assert!(
        Config::WORLD_HALF_SIZE > 100.0,
        "WORLD_HALF_SIZE should be at least 100, got {}",
        Config::WORLD_HALF_SIZE
    );
}

#[test]
fn config_insect_count_is_reasonable() {
    assert!(
        Config::INSECT_COUNT >= 50,
        "INSECT_COUNT should be at least 50, got {}",
        Config::INSECT_COUNT
    );
}

#[test]
fn config_bird_count_is_reasonable() {
    assert!(
        Config::BIRD_COUNT >= 5,
        "BIRD_COUNT should be at least 5, got {}",
        Config::BIRD_COUNT
    );
}

#[test]
fn config_insect_speed_is_positive() {
    assert!(Config::INSECT_SPEED > 0.0);
}

#[test]
fn config_bird_speed_faster_than_insects() {
    assert!(
        Config::BIRD_SPEED > Config::INSECT_SPEED,
        "Birds should be faster than insects: bird={} insect={}",
        Config::BIRD_SPEED,
        Config::INSECT_SPEED
    );
}

#[test]
fn config_bird_sight_range_positive() {
    assert!(Config::BIRD_SIGHT_RANGE > 0.0);
}

#[test]
fn config_bird_sight_angle_in_valid_range() {
    assert!(Config::BIRD_SIGHT_HALF_ANGLE > 0.0);
    assert!(
        Config::BIRD_SIGHT_HALF_ANGLE < std::f32::consts::PI,
        "Sight half angle should be less than PI (180°)"
    );
}

#[test]
fn config_flocking_weights_positive() {
    assert!(Config::SEPARATION_WEIGHT > 0.0);
    assert!(Config::ALIGNMENT_WEIGHT > 0.0);
    assert!(Config::COHESION_WEIGHT > 0.0);
}

#[test]
fn config_spatial_cell_size_positive() {
    assert!(Config::SPATIAL_CELL_SIZE > 0.0);
}

#[test]
fn config_respawn_threshold_less_than_initial_count() {
    assert!(
        Config::MIN_INSECT_COUNT < Config::INSECT_COUNT,
        "MIN_INSECT_COUNT ({}) should be less than INSECT_COUNT ({})",
        Config::MIN_INSECT_COUNT,
        Config::INSECT_COUNT
    );
}

#[test]
fn config_colors_insects_are_warm_and_visible() {
    // Insects should be warm-toned (golden amber) and bright enough to see
    let color = Colors::INSECT.to_srgba();
    let brightness = color.red + color.green + color.blue;
    assert!(
        brightness > 1.0,
        "Insect color should be bright enough to see, got brightness={}",
        brightness
    );
}

#[test]
fn config_colors_insects_and_birds_are_distinct() {
    let insect = Colors::INSECT.to_srgba();
    let bird = Colors::BIRD.to_srgba();
    let diff = (insect.red - bird.red).abs()
        + (insect.green - bird.green).abs()
        + (insect.blue - bird.blue).abs();
    assert!(
        diff > 0.5,
        "Insect and bird colors should be visually distinct, got diff={}",
        diff
    );
}

// =============================================================
// Spatial index tests — verify insert and neighbor queries
// =============================================================

#[test]
fn spatial_index_insert_and_retrieve_nearby() {
    let mut index = SpatialIndex::new(10.0);
    let entity = Entity::from_raw_u32(1).unwrap();
    index.insert(entity, Vec3::new(5.0, 5.0, 5.0));

    let nearby = index.get_nearby(Vec3::new(5.0, 5.0, 5.0));
    assert!(
        nearby.contains(&entity),
        "Entity inserted at (5,5,5) should be found near (5,5,5)"
    );
}

#[test]
fn spatial_index_finds_entity_in_adjacent_cell() {
    let mut index = SpatialIndex::new(10.0);
    let entity = Entity::from_raw_u32(1).unwrap();
    // Insert at position just inside cell (0,0,0)
    index.insert(entity, Vec3::new(1.0, 1.0, 1.0));

    // Query from adjacent cell (1,0,0) — position 15.0 is in cell 1
    let nearby = index.get_nearby(Vec3::new(15.0, 1.0, 1.0));
    assert!(
        nearby.contains(&entity),
        "Entity in adjacent cell should be found via get_nearby"
    );
}

#[test]
fn spatial_index_does_not_find_distant_entity() {
    let mut index = SpatialIndex::new(10.0);
    let entity = Entity::from_raw_u32(1).unwrap();
    index.insert(entity, Vec3::new(0.0, 0.0, 0.0));

    // Query from far away — cell (5,5,5) is not adjacent to (0,0,0)
    let nearby = index.get_nearby(Vec3::new(50.0, 50.0, 50.0));
    assert!(
        !nearby.contains(&entity),
        "Entity at origin should NOT be found near (50,50,50)"
    );
}

#[test]
fn spatial_index_clear_removes_all_entities() {
    let mut index = SpatialIndex::new(10.0);
    index.insert(Entity::from_raw_u32(1).unwrap(), Vec3::new(5.0, 5.0, 5.0));
    index.insert(Entity::from_raw_u32(2).unwrap(), Vec3::new(15.0, 15.0, 15.0));

    index.clear();

    let nearby = index.get_nearby(Vec3::new(5.0, 5.0, 5.0));
    assert!(nearby.is_empty(), "After clear, no entities should be found");
}

#[test]
fn spatial_index_handles_negative_positions() {
    let mut index = SpatialIndex::new(10.0);
    let entity = Entity::from_raw_u32(1).unwrap();
    index.insert(entity, Vec3::new(-5.0, -5.0, -5.0));

    let nearby = index.get_nearby(Vec3::new(-5.0, -5.0, -5.0));
    assert!(
        nearby.contains(&entity),
        "Entity at negative position should be retrievable"
    );
}

#[test]
fn spatial_index_multiple_entities_same_cell() {
    let mut index = SpatialIndex::new(10.0);
    let e1 = Entity::from_raw_u32(1).unwrap();
    let e2 = Entity::from_raw_u32(2).unwrap();
    index.insert(e1, Vec3::new(1.0, 1.0, 1.0));
    index.insert(e2, Vec3::new(2.0, 2.0, 2.0));

    let nearby = index.get_nearby(Vec3::new(1.0, 1.0, 1.0));
    assert!(nearby.contains(&e1));
    assert!(nearby.contains(&e2));
}

// =============================================================
// Component construction tests — verify components hold data correctly
// =============================================================

#[test]
fn velocity_stores_vec3() {
    let v = Velocity(Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(v.0, Vec3::new(1.0, 2.0, 3.0));
}

#[test]
fn brownian_motion_stores_wander_strength() {
    let bm = BrownianMotion {
        wander_strength: 2.5,
    };
    assert_eq!(bm.wander_strength, 2.5);
}

#[test]
fn predator_stores_sight_params() {
    let p = Predator {
        sight_range: 80.0,
        sight_half_angle: 0.7,
    };
    assert_eq!(p.sight_range, 80.0);
    assert_eq!(p.sight_half_angle, 0.7);
}

#[test]
fn flocking_stores_weights() {
    let f = Flocking {
        separation_weight: 1.5,
        alignment_weight: 1.0,
        cohesion_weight: 1.0,
    };
    assert_eq!(f.separation_weight, 1.5);
    assert_eq!(f.alignment_weight, 1.0);
    assert_eq!(f.cohesion_weight, 1.0);
}
