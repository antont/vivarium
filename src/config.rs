use bevy::prelude::*;

/// Simulation scale factor, read from `SIM_SCALE` env var (default 1.0).
/// Multiplies entity counts and world size for stress-testing.
#[derive(Resource, Clone, Copy)]
pub struct SimScale(pub f32);

impl Default for SimScale {
    fn default() -> Self {
        let scale = std::env::var("SIM_SCALE")
            .ok()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(1.0)
            .max(0.1);
        Self(scale)
    }
}

impl SimScale {
    pub fn count(&self, base: usize) -> usize {
        ((base as f32) * self.0).round() as usize
    }
    pub fn size(&self, base: f32) -> f32 {
        base * self.0.sqrt()
    }
}

pub struct Config;

impl Config {
    // World
    pub const WORLD_HALF_SIZE: f32 = 200.0;

    // Insects
    pub const INSECT_COUNT: usize = 200;
    pub const INSECT_SPEED: f32 = 8.0;
    pub const INSECT_WANDER_STRENGTH: f32 = 45.0; // max degrees rotation per frame
    pub const INSECT_RADIUS: f32 = 1.2;

    // Birds
    pub const BIRD_COUNT: usize = 20;
    pub const BIRD_SPEED: f32 = 60.0;
    pub const BIRD_SIGHT_RANGE: f32 = 80.0;
    pub const BIRD_SIGHT_HALF_ANGLE: f32 = 0.7; // ~40 degrees
    pub const BIRD_RADIUS: f32 = 5.0;
    pub const BIRD_EATING_DISTANCE: f32 = 3.0;

    // Flocking
    pub const FLOCK_NEIGHBOR_RADIUS: f32 = 40.0;
    pub const SEPARATION_WEIGHT: f32 = 1.5;
    pub const ALIGNMENT_WEIGHT: f32 = 1.0;
    pub const COHESION_WEIGHT: f32 = 1.0;
    pub const SEPARATION_DISTANCE: f32 = 10.0;

    // Insect swarm cohesion
    pub const SWARM_COHESION_RADIUS: f32 = 30.0;
    pub const SWARM_COHESION_WEIGHT: f32 = 0.5;

    // Bird hunting
    pub const HUNT_CIRCLE_DURATION: f32 = 1.5; // seconds circling before dive
    pub const HUNT_CIRCLE_RADIUS: f32 = 25.0;
    pub const HUNT_DIVE_SPEED_MULT: f32 = 1.8; // speed multiplier during dive
    pub const HUNT_DIVE_DISTANCE: f32 = 8.0; // close enough to switch to dive

    // Bird wandering
    pub const BIRD_WANDER_STRENGTH: f32 = 15.0; // degrees per frame when searching

    // Wind
    pub const WIND_MAX_STRENGTH: f32 = 6.0; // max displacement speed (units/s)
    pub const WIND_BASE_STRENGTH: f32 = 2.0; // average wind speed
    pub const WIND_INSECT_FACTOR: f32 = 1.0; // insects feel full wind
    pub const WIND_BIRD_FACTOR: f32 = 0.15; // birds barely affected
    pub const WIND_TREE_BEND_FACTOR: f32 = 0.004; // radians per unit of wind strength
    pub const WIND_DIR_RATE: f32 = 0.1; // direction oscillation rate (rad/s)
    pub const WIND_STR_RATE: f32 = 0.08; // strength oscillation rate (rad/s)

    // Squirrel
    pub const SQUIRREL_COUNT: usize = 3;
    pub const SQUIRREL_GROUND_SPEED: f32 = 15.0;
    pub const SQUIRREL_CLIMB_SPEED: f32 = 8.0;
    pub const SQUIRREL_IDLE_MIN: f32 = 1.0;
    pub const SQUIRREL_IDLE_MAX: f32 = 3.0;
    pub const SQUIRREL_BODY_SCALE: f32 = 5.0;

    // Nesting
    pub const NEST_BUILD_TIME: f32 = 4.0;
    pub const EGG_HATCH_TIME: f32 = 12.0;
    pub const NEST_ARRIVAL_DISTANCE: f32 = 5.0;
    pub const BIRD_DEFEND_SPEED_MULT: f32 = 2.0;

    // Squirrel predation
    pub const SQUIRREL_HATCHLING_SIGHT_RANGE: f32 = 60.0;
    pub const HATCHLING_ALERT_RADIUS: f32 = 20.0;

    // Navigation
    pub const NAV_GROUND_SPACING: f32 = 40.0;

    // Boundary force field
    pub const BOUNDARY_MARGIN: f32 = 40.0; // distance from edge where force kicks in
    pub const BOUNDARY_FORCE: f32 = 5.0; // steering strength

    // Spatial index
    pub const SPATIAL_CELL_SIZE: f32 = 40.0;

    // Respawn
    pub const MIN_INSECT_COUNT: usize = 150;
    pub const INSECT_RESPAWN_BATCH: usize = 10;
}

pub struct Colors;

impl Colors {
    pub const INSECT: Color = Color::srgb(0.15, 0.15, 0.15); // dark but visible
    pub const BIRD: Color = Color::srgb(0.92, 0.92, 0.96); // bright near-white
    pub const BACKGROUND: Color = Color::srgb(0.65, 0.8, 0.95); // light blue sky
    pub const GROUND: Color = Color::srgb(0.45, 0.52, 0.32); // warmer sage green
    pub const SQUIRREL_FUR: Color = Color::srgb(0.8, 0.35, 0.1); // orange-red
    pub const SQUIRREL_DARK: Color = Color::srgb(0.35, 0.15, 0.05); // dark rust
    pub const NEST: Color = Color::srgb(0.4, 0.25, 0.1); // brown
    pub const HATCHLING: Color = Color::srgb(0.9, 0.85, 0.3); // yellow
}
