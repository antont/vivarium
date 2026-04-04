use bevy::prelude::*;

/// Marker for insect entities.
#[derive(Component, Default)]
pub struct Insect;

/// Marker for bird entities.
#[derive(Component, Default)]
pub struct Bird;

/// 3D velocity vector.
#[derive(Component)]
pub struct Velocity(pub Vec3);

/// Drives Brownian motion wandering behavior.
#[derive(Component)]
pub struct BrownianMotion {
    pub wander_strength: f32,
}

/// Cone-of-sight predator detection.
#[derive(Component)]
pub struct Predator {
    pub sight_range: f32,
    pub sight_half_angle: f32,
}

/// Boids flocking behavior parameters.
#[derive(Component, Clone)]
pub struct Flocking {
    pub separation_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,
}

/// Marker: entity wraps at world boundaries.
#[derive(Component)]
pub struct BoundaryWrap;

/// Swarm cohesion: steers toward nearby same-type entities.
#[derive(Component, Clone)]
pub struct SwarmCohesion {
    pub radius: f32,
    pub weight: f32,
}

/// Bird hunting state machine.
#[derive(Component)]
pub struct HuntState {
    pub phase: HuntPhase,
    pub timer: f32,
    pub target_pos: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HuntPhase {
    Searching,
    Circling,
    Diving,
}

impl Default for HuntState {
    fn default() -> Self {
        Self {
            phase: HuntPhase::Searching,
            timer: 0.0,
            target_pos: Vec3::ZERO,
        }
    }
}

/// Gentle wandering for birds when not hunting.
#[derive(Component)]
pub struct Wander {
    pub strength: f32,
}
