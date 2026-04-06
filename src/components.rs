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

/// Bird nesting lifecycle — layered above HuntState.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BirdLifecycle {
    Hunting,        // normal hunt behavior
    FlyingToNest,   // steering toward branch node
    Building,       // sitting at branch, building nest (timer)
    HuntingForEgg,  // hunting an insect to return and lay egg
    Incubating,     // sitting on egg (timer)
    Parenting,      // has hatchling, hunts and delivers food
    Defending,      // alert received, flying fast to nest
}

#[derive(Component)]
pub struct BirdNestingState {
    pub phase: BirdLifecycle,
    pub nest: Option<Entity>,
    pub nest_nav_node: Option<usize>,
    pub timer: f32,
    pub insects_eaten: u32,
}

impl Default for BirdNestingState {
    fn default() -> Self {
        Self {
            phase: BirdLifecycle::Hunting,
            nest: None,
            nest_nav_node: None,
            timer: 0.0,
            insects_eaten: 0,
        }
    }
}

/// A nest on a tree branch.
#[derive(Component)]
pub struct Nest {
    pub parent_bird: Entity,
    pub nav_node: usize,
    pub has_egg: bool,
    pub has_hatchling: bool,
}

/// A baby bird that stays at its nest.
#[derive(Component)]
pub struct Hatchling {
    pub nest: Entity,
    pub parent_bird: Entity,
    pub alert: bool,
}

/// Message: a bird ate an insect.
#[derive(bevy::ecs::message::Message)]
pub struct InsectEaten {
    pub bird: Entity,
}

/// Marker for squirrel entities.
#[derive(Component)]
pub struct Squirrel;

/// Identifies which squirrel this is (0, 1, 2, …) for focus cameras.
#[derive(Component)]
pub struct SquirrelIndex(pub usize);

/// Squirrel is targeting a hatchling.
#[derive(Component)]
pub struct SquirrelTarget {
    pub hatchling: Entity,
    pub nest_nav_node: usize,
}

/// Squirrel behavior state machine.
#[derive(Component)]
pub struct SquirrelState {
    pub phase: SquirrelPhase,
    pub path: Vec<usize>,      // nav node indices
    pub path_index: usize,     // current position in path
    pub progress: f32,         // 0..1 lerp between current and next node
    pub timer: f32,
    pub last_normal: Vec3,     // previous frame's surface normal for stable projection
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SquirrelPhase {
    Idle,
    Moving,
    Hunting,  // navigating toward a hatchling
    Fleeing,  // parent arrived, running away
}

impl Default for SquirrelState {
    fn default() -> Self {
        Self {
            phase: SquirrelPhase::Idle,
            path: Vec::new(),
            path_index: 0,
            progress: 0.0,
            timer: 0.0,
            last_normal: Vec3::Y,
        }
    }
}
