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

/// Marker for tree segment entities affected by wind.
#[derive(Component)]
pub struct TreeSegment;

/// Stores the spawn-time local rotation so wind tilt can be added on top.
#[derive(Component)]
pub struct BaseLocalRotation(pub Quat);

/// Marker for squirrel entities.
#[derive(Component)]
pub struct Squirrel;

/// Squirrel behavior state machine.
#[derive(Component)]
pub struct SquirrelState {
    pub phase: SquirrelPhase,
    pub path: Vec<usize>,      // nav node indices
    pub path_index: usize,     // current position in path
    pub progress: f32,         // 0..1 lerp between current and next node
    pub timer: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SquirrelPhase {
    Idle,
    Moving,
}

impl Default for SquirrelState {
    fn default() -> Self {
        Self {
            phase: SquirrelPhase::Idle,
            path: Vec::new(),
            path_index: 0,
            progress: 0.0,
            timer: 0.0,
        }
    }
}
