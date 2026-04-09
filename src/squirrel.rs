use bevy::prelude::*;
use rand::Rng;
use crate::components::{BirdNestingState, Bird, Hatchling, Squirrel, SquirrelIndex, SquirrelPhase, SquirrelState, SquirrelTarget};
use crate::config::Config;
use crate::nav_graph::{NavGraph, NavNodeKind};

/// Spawn a squirrel at a given position (logic only — visual added by squirrel_visual_system).
pub fn spawn_squirrel(
    commands: &mut Commands,
    position: Vec3,
    index: usize,
) -> Entity {
    commands
        .spawn((
            Squirrel,
            SquirrelIndex(index),
            SquirrelState::default(),
            Transform::from_translation(position),
            Visibility::default(),
        ))
        .id()
}

/// Squirrel behavior: pick targets and manage state transitions.
pub fn squirrel_behavior_system(
    time: Res<Time>,
    nav: Res<NavGraph>,
    mut squirrels: Query<(&Transform, &mut SquirrelState), With<Squirrel>>,
) {
    let dt = time.delta_secs();
    let mut rng = rand::rng();

    for (transform, mut state) in &mut squirrels {
        match state.phase {
            SquirrelPhase::Idle => {
                state.timer -= dt;
                if state.timer <= 0.0 {
                    // Pick a random target node
                    if nav.nodes.is_empty() {
                        continue;
                    }
                    let target = rng.random_range(0..nav.nodes.len());
                    let current = nav.nearest_node(transform.translation).unwrap_or(0);

                    if current == target {
                        state.timer = rng.random_range(Config::SQUIRREL_IDLE_MIN..Config::SQUIRREL_IDLE_MAX);
                        continue;
                    }

                    if let Some(path) = nav.find_path(current, target) {
                        if path.len() > 1 {
                            state.path = path;
                            state.path_index = 0;
                            state.progress = 0.0;
                            state.phase = SquirrelPhase::Moving;
                        }
                    } else {
                        // No path found, try again soon
                        state.timer = 0.5;
                    }
                }
            }
            SquirrelPhase::Moving => {
                // Handled in movement system; check if path is done
                if state.path_index >= state.path.len().saturating_sub(1) {
                    state.phase = SquirrelPhase::Idle;
                    state.timer = rng.random_range(Config::SQUIRREL_IDLE_MIN..Config::SQUIRREL_IDLE_MAX);
                    state.path.clear();
                }
            }
            SquirrelPhase::Hunting => {
                // Same as Moving — path leads to hatchling's nest
                if state.path_index >= state.path.len().saturating_sub(1) {
                    // Arrived at nest — stay briefly, then go idle
                    state.phase = SquirrelPhase::Idle;
                    state.timer = rng.random_range(Config::SQUIRREL_IDLE_MIN..Config::SQUIRREL_IDLE_MAX);
                    state.path.clear();
                }
            }
            SquirrelPhase::Fleeing => {
                // Same as Moving — path leads away from nest
                if state.path_index >= state.path.len().saturating_sub(1) {
                    state.phase = SquirrelPhase::Idle;
                    state.timer = rng.random_range(Config::SQUIRREL_IDLE_MIN..Config::SQUIRREL_IDLE_MAX);
                    state.path.clear();
                }
            }
        }
    }
}

/// Idle squirrels scan for nearby hatchlings and pathfind toward them.
pub fn squirrel_hatchling_detection_system(
    mut commands: Commands,
    nav: Res<NavGraph>,
    mut squirrels: Query<(Entity, &Transform, &mut SquirrelState), (With<Squirrel>, Without<SquirrelTarget>)>,
    hatchlings: Query<(Entity, &Transform, &Hatchling)>,
) {
    for (sq_entity, sq_transform, mut state) in &mut squirrels {
        if state.phase != SquirrelPhase::Idle {
            continue;
        }

        // Find closest hatchling within sight range
        let mut closest: Option<(Entity, f32, usize)> = None;
        for (h_entity, h_transform, _hatchling) in &hatchlings {
            let dist = sq_transform.translation.distance(h_transform.translation);
            if dist < Config::SQUIRREL_HATCHLING_SIGHT_RANGE {
                let nav_node = nav.nearest_node(h_transform.translation).unwrap_or(0);
                if closest.is_none() || dist < closest.unwrap().1 {
                    closest = Some((h_entity, dist, nav_node));
                }
            }
        }

        if let Some((h_entity, _, nest_nav_node)) = closest {
            let current = nav.nearest_node(sq_transform.translation).unwrap_or(0);
            if let Some(path) = nav.find_path(current, nest_nav_node) {
                if path.len() > 1 {
                    state.path = path;
                    state.path_index = 0;
                    state.progress = 0.0;
                    state.phase = SquirrelPhase::Hunting;
                    commands.entity(sq_entity).insert(SquirrelTarget {
                        hatchling: h_entity,
                        nest_nav_node,
                    });
                }
            }
        }
    }
}

/// When parent bird arrives at nest, nearby hunting squirrels flee.
pub fn squirrel_flee_system(
    mut commands: Commands,
    nav: Res<NavGraph>,
    birds: Query<(&Transform, &BirdNestingState), With<Bird>>,
    mut squirrels: Query<(Entity, &Transform, &mut SquirrelState, &SquirrelTarget), With<Squirrel>>,
) {
    let mut rng = rand::rng();

    for (sq_entity, sq_transform, mut state, target) in &mut squirrels {
        if state.phase != SquirrelPhase::Hunting {
            continue;
        }

        // Check if parent bird is near the nest
        let parent_nearby = birds.iter().any(|(bird_transform, nesting)| {
            if let Some(nav_node) = nesting.nest_nav_node {
                if nav_node == target.nest_nav_node {
                    let dist = bird_transform.translation.distance(sq_transform.translation);
                    return dist < Config::HATCHLING_ALERT_RADIUS * 2.0;
                }
            }
            false
        });

        if parent_nearby {
            // Flee: pick a random ground node far away
            let ground_nodes: Vec<usize> = nav.nodes.iter().enumerate()
                .filter(|(_, n)| n.kind == NavNodeKind::Ground)
                .map(|(i, _)| i)
                .collect();

            if !ground_nodes.is_empty() {
                let flee_target = ground_nodes[rng.random_range(0..ground_nodes.len())];
                let current = nav.nearest_node(sq_transform.translation).unwrap_or(0);
                if let Some(path) = nav.find_path(current, flee_target) {
                    if path.len() > 1 {
                        state.path = path;
                        state.path_index = 0;
                        state.progress = 0.0;
                        state.phase = SquirrelPhase::Fleeing;
                    }
                }
            }
            commands.entity(sq_entity).remove::<SquirrelTarget>();
        }
    }
}

/// Project a point onto the surface of a cylinder defined by two endpoints and a radius.
/// `hint_normal` biases the projection direction when the point is on/near the axis,
/// preventing flicker from the radial direction flipping every frame.
/// Returns (surface_point, surface_normal).
pub fn project_to_cylinder(
    point: Vec3,
    axis_start: Vec3,
    axis_end: Vec3,
    radius: f32,
    hint_normal: Option<Vec3>,
) -> (Vec3, Vec3) {
    let axis = axis_end - axis_start;
    let axis_len = axis.length();
    if axis_len < f32::EPSILON {
        return (point, hint_normal.unwrap_or(Vec3::Y));
    }
    let axis_dir = axis / axis_len;

    // Project point onto axis
    let to_point = point - axis_start;
    let t = to_point.dot(axis_dir).clamp(0.0, axis_len);
    let closest_on_axis = axis_start + axis_dir * t;

    // Radial direction from axis to point
    let radial = point - closest_on_axis;
    let radial_len = radial.length();

    // Threshold: if within 10% of radius, the radial direction is unstable — use hint
    let stable_threshold = radius * 0.1;
    let normal = if radial_len > stable_threshold {
        radial / radial_len
    } else if let Some(hint) = hint_normal {
        // Project hint onto the plane perpendicular to axis (remove axis component)
        let projected = hint - axis_dir * hint.dot(axis_dir);
        let proj_len = projected.length();
        if proj_len > f32::EPSILON {
            projected / proj_len
        } else {
            fallback_perpendicular(axis_dir)
        }
    } else {
        fallback_perpendicular(axis_dir)
    };

    let surface_point = closest_on_axis + normal * radius;
    (surface_point, normal)
}

fn fallback_perpendicular(axis_dir: Vec3) -> Vec3 {
    if axis_dir.dot(Vec3::X).abs() < 0.9 {
        axis_dir.cross(Vec3::X).normalize()
    } else {
        axis_dir.cross(Vec3::Z).normalize()
    }
}

/// Squirrel movement: interpolate along nav path edges, project to surface.
pub fn squirrel_movement_system(
    time: Res<Time>,
    nav: Res<NavGraph>,
    global_transforms: Query<&GlobalTransform>,
    mut squirrels: Query<(&mut Transform, &mut SquirrelState), With<Squirrel>>,
) {
    let dt = time.delta_secs();
    let ground_y = -Config::WORLD_HALF_SIZE;

    for (mut transform, mut state) in &mut squirrels {
        let is_moving = matches!(state.phase, SquirrelPhase::Moving | SquirrelPhase::Hunting | SquirrelPhase::Fleeing);
        if !is_moving || state.path.len() < 2 || state.path_index + 1 >= state.path.len() {
            continue;
        }

        let current_idx = state.path[state.path_index];
        let next_idx = state.path[state.path_index + 1];
        let current_pos = nav.node_position(current_idx, &global_transforms);
        let next_pos = nav.node_position(next_idx, &global_transforms);

        let edge_length = current_pos.distance(next_pos);
        if edge_length < f32::EPSILON {
            state.path_index += 1;
            state.progress = 0.0;
            continue;
        }

        // Speed depends on whether climbing (vertical) or running (horizontal)
        let direction = next_pos - current_pos;
        let vertical_ratio = direction.y.abs() / edge_length;
        let speed = Config::SQUIRREL_GROUND_SPEED * (1.0 - vertical_ratio * 0.5);

        state.progress += (speed * dt) / edge_length;

        if state.progress >= 1.0 {
            state.path_index += 1;
            state.progress = 0.0;
        }
        let t = state.progress.clamp(0.0, 1.0);
        let center_pos = current_pos.lerp(next_pos, t);

        // Surface projection: push squirrel to the surface of the branch or ground
        let current_node = &nav.nodes[current_idx];
        let next_node = &nav.nodes[next_idx];
        let both_on_branch = current_node.kind == NavNodeKind::Branch
            && next_node.kind == NavNodeKind::Branch;
        let either_on_branch = current_node.kind == NavNodeKind::Branch
            || next_node.kind == NavNodeKind::Branch;

        let hint = if state.last_normal != Vec3::ZERO { Some(state.last_normal) } else { None };

        let (surface_pos, surface_normal) = if both_on_branch {
            // Moving along a branch — project to cylinder surface
            let radius = current_node.radius.max(next_node.radius);
            project_to_cylinder(center_pos, current_pos, next_pos, radius, hint)
        } else if either_on_branch {
            // Transitioning between ground and branch — lerp radius
            let branch_radius = current_node.radius.max(next_node.radius);
            let branch_factor = if current_node.kind == NavNodeKind::Branch {
                1.0 - t
            } else {
                t
            };
            let radius = branch_radius * branch_factor;
            if radius > 0.5 {
                project_to_cylinder(center_pos, current_pos, next_pos, radius, hint)
            } else {
                (Vec3::new(center_pos.x, ground_y, center_pos.z), Vec3::Y)
            }
        } else {
            // On the ground
            (Vec3::new(center_pos.x, ground_y, center_pos.z), Vec3::Y)
        };

        state.last_normal = surface_normal;
        transform.translation = surface_pos;

        // Orient squirrel: "up" = surface normal, face movement direction
        let move_dir = (next_pos - current_pos).normalize_or_zero();
        if move_dir != Vec3::ZERO && surface_normal != Vec3::ZERO {
            // Project movement direction onto the surface plane
            let tangent = (move_dir - surface_normal * move_dir.dot(surface_normal)).normalize_or_zero();
            let tangent = if tangent == Vec3::ZERO { move_dir } else { tangent };
            let look_rot = Transform::default()
                .looking_to(-tangent, surface_normal)
                .rotation;
            transform.rotation = transform.rotation.slerp(look_rot, 8.0 * dt);
        }
    }
}
