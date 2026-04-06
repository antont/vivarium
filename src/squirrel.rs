use bevy::prelude::*;
use rand::Rng;
use crate::components::{Squirrel, SquirrelPhase, SquirrelState};
use crate::config::Config;
use crate::nav_graph::{NavGraph, NavNodeKind};

/// Spawn a squirrel at a given position with multi-primitive body.
pub fn spawn_squirrel(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) {
    let s = Config::SQUIRREL_BODY_SCALE;

    let fur = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.3, 0.15), // warm reddish-brown
        unlit: true,
        ..default()
    });
    let dark = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.1, 0.05),
        unlit: true,
        ..default()
    });

    // Root entity with behavior
    let root = commands
        .spawn((
            Squirrel,
            SquirrelState::default(),
            Transform::from_translation(position),
            Visibility::default(),
        ))
        .id();

    // Body — stretched sphere
    let body = commands
        .spawn((
            Mesh3d(meshes.add(Sphere::new(s * 0.5))),
            MeshMaterial3d(fur.clone()),
            Transform::from_scale(Vec3::new(0.7, 0.5, 1.0)),
        ))
        .id();
    commands.entity(root).add_child(body);

    // Head — smaller sphere, forward and up
    let head = commands
        .spawn((
            Mesh3d(meshes.add(Sphere::new(s * 0.3))),
            MeshMaterial3d(fur.clone()),
            Transform::from_translation(Vec3::new(0.0, s * 0.25, s * 0.6)),
        ))
        .id();
    commands.entity(root).add_child(head);

    // Ears — two tiny spheres
    for side in [-1.0_f32, 1.0] {
        let ear = commands
            .spawn((
                Mesh3d(meshes.add(Sphere::new(s * 0.1))),
                MeshMaterial3d(dark.clone()),
                Transform::from_translation(Vec3::new(
                    side * s * 0.2,
                    s * 0.45,
                    s * 0.55,
                )),
            ))
            .id();
        commands.entity(root).add_child(ear);
    }

    // Tail — pivot at body attachment, then offset the mesh so it extends upward/back
    let tail_pivot = commands
        .spawn(Transform::from_translation(Vec3::new(0.0, s * 0.1, -s * 0.3))
            .with_rotation(Quat::from_axis_angle(Vec3::X, -0.8)))
        .id();
    commands.entity(root).add_child(tail_pivot);

    let tail_mesh = commands
        .spawn((
            Mesh3d(meshes.add(Sphere::new(s * 0.35))),
            MeshMaterial3d(fur.clone()),
            Transform::from_translation(Vec3::new(0.0, s * 0.4, 0.0))
                .with_scale(Vec3::new(0.4, 1.2, 0.5)),
        ))
        .id();
    commands.entity(tail_pivot).add_child(tail_mesh);

    // Feet — 4 tiny spheres
    for &(x, z) in &[(-0.3, 0.3), (0.3, 0.3), (-0.3, -0.2), (0.3, -0.2)] {
        let foot = commands
            .spawn((
                Mesh3d(meshes.add(Sphere::new(s * 0.1))),
                MeshMaterial3d(dark.clone()),
                Transform::from_translation(Vec3::new(x * s, -s * 0.35, z * s)),
            ))
            .id();
        commands.entity(root).add_child(foot);
    }
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
        if state.phase != SquirrelPhase::Moving || state.path.len() < 2 {
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
