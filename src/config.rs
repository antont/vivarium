use bevy::prelude::*;

pub struct Config;

impl Config {
    // World
    pub const WORLD_HALF_SIZE: f32 = 0.0; // WRONG: should be 200.0

    // Insects
    pub const INSECT_COUNT: usize = 0; // WRONG
    pub const INSECT_SPEED: f32 = 0.0; // WRONG
    pub const INSECT_WANDER_STRENGTH: f32 = 0.0; // WRONG
    pub const INSECT_RADIUS: f32 = 0.0; // WRONG

    // Birds
    pub const BIRD_COUNT: usize = 0; // WRONG
    pub const BIRD_SPEED: f32 = 0.0; // WRONG
    pub const BIRD_SIGHT_RANGE: f32 = 0.0; // WRONG
    pub const BIRD_SIGHT_HALF_ANGLE: f32 = 0.0; // WRONG
    pub const BIRD_RADIUS: f32 = 0.0; // WRONG
    pub const BIRD_EATING_DISTANCE: f32 = 0.0; // WRONG

    // Flocking
    pub const FLOCK_NEIGHBOR_RADIUS: f32 = 0.0; // WRONG
    pub const SEPARATION_WEIGHT: f32 = 0.0; // WRONG
    pub const ALIGNMENT_WEIGHT: f32 = 0.0; // WRONG
    pub const COHESION_WEIGHT: f32 = 0.0; // WRONG
    pub const SEPARATION_DISTANCE: f32 = 0.0; // WRONG

    // Spatial index
    pub const SPATIAL_CELL_SIZE: f32 = 0.0; // WRONG

    // Respawn
    pub const MIN_INSECT_COUNT: usize = 0; // WRONG
    pub const INSECT_RESPAWN_BATCH: usize = 0; // WRONG
}

pub struct Colors;

impl Colors {
    pub const INSECT: Color = Color::srgb(0.0, 0.0, 0.0); // WRONG
    pub const BIRD: Color = Color::srgb(0.0, 0.0, 0.0); // WRONG
}
