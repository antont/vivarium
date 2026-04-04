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
#[derive(Component)]
pub struct Flocking {
    pub separation_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,
}

/// Marker: entity wraps at world boundaries.
#[derive(Component)]
pub struct BoundaryWrap;
