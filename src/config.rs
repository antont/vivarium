use bevy::prelude::*;

pub struct Config;

impl Config {
    // World
    pub const WORLD_HALF_SIZE: f32 = 200.0;

    // Insects
    pub const INSECT_COUNT: usize = 200;
    pub const INSECT_SPEED: f32 = 30.0;
    pub const INSECT_WANDER_STRENGTH: f32 = 5.0; // max degrees rotation per frame
    pub const INSECT_RADIUS: f32 = 0.3;

    // Birds
    pub const BIRD_COUNT: usize = 20;
    pub const BIRD_SPEED: f32 = 60.0;
    pub const BIRD_SIGHT_RANGE: f32 = 80.0;
    pub const BIRD_SIGHT_HALF_ANGLE: f32 = 0.7; // ~40 degrees
    pub const BIRD_RADIUS: f32 = 1.0;
    pub const BIRD_EATING_DISTANCE: f32 = 3.0;

    // Flocking
    pub const FLOCK_NEIGHBOR_RADIUS: f32 = 40.0;
    pub const SEPARATION_WEIGHT: f32 = 1.5;
    pub const ALIGNMENT_WEIGHT: f32 = 1.0;
    pub const COHESION_WEIGHT: f32 = 1.0;
    pub const SEPARATION_DISTANCE: f32 = 10.0;

    // Boundary force field
    pub const BOUNDARY_MARGIN: f32 = 40.0;  // distance from edge where force kicks in
    pub const BOUNDARY_FORCE: f32 = 5.0;    // steering strength

    // Spatial index
    pub const SPATIAL_CELL_SIZE: f32 = 40.0;

    // Respawn
    pub const MIN_INSECT_COUNT: usize = 150;
    pub const INSECT_RESPAWN_BATCH: usize = 10;
}

pub struct Colors;

impl Colors {
    pub const INSECT: Color = Color::srgb(0.2, 0.8, 0.2); // green
    pub const BIRD: Color = Color::srgb(0.9, 0.4, 0.1); // orange
}
