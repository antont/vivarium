use bevy::prelude::*;
use rand::Rng;
use crate::components::{Squirrel, SquirrelPhase, SquirrelState};
use crate::config::Config;
use crate::nav_graph::NavGraph;

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

    // Tail — stretched sphere, angled up and back
    let tail = commands
        .spawn((
            Mesh3d(meshes.add(Sphere::new(s * 0.35))),
            MeshMaterial3d(fur.clone()),
            Transform::from_translation(Vec3::new(0.0, s * 0.5, -s * 0.7))
                .with_scale(Vec3::new(0.4, 0.5, 1.2)),
        ))
        .id();
    commands.entity(root).add_child(tail);

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

/// Squirrel movement: interpolate along nav path edges, following live tree positions.
pub fn squirrel_movement_system(
    time: Res<Time>,
    nav: Res<NavGraph>,
    global_transforms: Query<&GlobalTransform>,
    mut squirrels: Query<(&mut Transform, &mut SquirrelState), With<Squirrel>>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut state) in &mut squirrels {
        if state.phase != SquirrelPhase::Moving || state.path.len() < 2 {
            continue;
        }

        let current_idx = state.path[state.path_index];
        let next_idx = state.path[state.path_index + 1];
        // Use live entity positions for branch nodes (tracks wind bending)
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
            transform.translation = next_pos;
        } else {
            transform.translation = current_pos.lerp(next_pos, state.progress);
        }

        // Face movement direction
        let move_dir = (next_pos - transform.translation).normalize_or_zero();
        if move_dir != Vec3::ZERO {
            // Point +Z toward movement direction
            let target_rot = Transform::default()
                .looking_to(move_dir, Vec3::Y)
                .rotation;
            transform.rotation = transform.rotation.slerp(target_rot, 5.0 * dt);
        }
    }
}
