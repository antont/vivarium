use bevy::prelude::*;
use rand::Rng;
use crate::components::{BaseLocalRotation, TreeSegment};
use crate::nav_graph::{NavGraph, NavNodeKind};

/// A simple L-system tree generator.
/// Rules: F → F[+F]F[-F]F  (branching pattern)
/// Symbols: F = forward, + = turn right, - = turn left, [ = push, ] = pop

struct LSystem {
    axiom: String,
    iterations: usize,
    angle: f32,  // branch angle in radians
    length: f32, // initial segment length
    shrink: f32, // length multiplier per generation
    radius: f32, // initial branch radius
    radius_shrink: f32,
}

impl LSystem {
    fn generate(&self) -> String {
        let mut current = self.axiom.clone();
        for _ in 0..self.iterations {
            let mut next = String::new();
            for ch in current.chars() {
                match ch {
                    'F' => next.push_str("F[+F]F[-F]F"),
                    c => next.push(c),
                }
            }
            current = next;
        }
        current
    }
}

/// Spawn a procedural L-system tree and register nav nodes in the graph.
pub fn spawn_tree(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    nav_graph: &mut NavGraph,
    position: Vec3,
    scale: f32,
) {
    let lsystem = LSystem {
        axiom: "F".to_string(),
        iterations: 3,
        angle: 1.2,
        length: 10.0 * scale,
        shrink: 0.65,
        radius: 2.0 * scale,
        radius_shrink: 0.6,
    };

    let instructions = lsystem.generate();

    let bark_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.25, 0.15),
        perceptual_roughness: 0.95,
        ..default()
    });

    let leaf_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.45, 0.15),
        perceptual_roughness: 0.8,
        ..default()
    });

    // Root anchor entity at tree base
    let root = commands.spawn(Transform::from_translation(position)).id();

    // Nav node at tree base
    let base_nav = nav_graph.add_node(position, NavNodeKind::TreeBase);

    let mut current_parent = root;
    let mut segment_length = lsystem.length;
    let mut segment_radius = lsystem.radius;
    let mut pending_rotation = Quat::IDENTITY;
    let mut rng = rand::rng();

    // Track world-space position and rotation for nav graph
    let mut world_pos = position;
    let mut world_rot = Quat::IDENTITY;
    let mut current_nav_node = base_nav;

    // Stack: (entity, length, radius, pending_rot, world_pos, world_rot, nav_node)
    let mut stack: Vec<(Entity, f32, f32, Quat, Vec3, Quat, usize)> = Vec::new();

    for ch in instructions.chars() {
        match ch {
            'F' => {
                let wobble_axis = Vec3::new(
                    rng.random_range(-0.1..0.1),
                    rng.random_range(-0.05..0.05),
                    rng.random_range(-0.1..0.1),
                )
                .normalize_or_zero();
                let wobble = if wobble_axis != Vec3::ZERO {
                    Quat::from_axis_angle(wobble_axis, 0.05)
                } else {
                    Quat::IDENTITY
                };

                let local_rotation = pending_rotation * wobble;
                pending_rotation = Quat::IDENTITY;

                // Update world-space tracking
                world_rot = world_rot * local_rotation;
                let tip_world = world_pos + world_rot * Vec3::new(0.0, segment_length, 0.0);

                // Spawn segment entity
                let segment = commands
                    .spawn((
                        TreeSegment,
                        BaseLocalRotation(local_rotation),
                        Transform::from_rotation(local_rotation),
                    ))
                    .id();
                commands.entity(current_parent).add_child(segment);

                // Cylinder mesh
                let mesh_child = commands
                    .spawn((
                        Mesh3d(meshes.add(Cylinder::new(segment_radius, segment_length))),
                        MeshMaterial3d(bark_material.clone()),
                        Transform::from_translation(Vec3::new(0.0, segment_length / 2.0, 0.0)),
                    ))
                    .id();
                commands.entity(segment).add_child(mesh_child);

                // Tip entity
                let tip = commands
                    .spawn(Transform::from_translation(Vec3::new(
                        0.0,
                        segment_length,
                        0.0,
                    )))
                    .id();
                commands.entity(segment).add_child(tip);

                current_parent = tip;

                // Add nav node at segment tip, linked to the tip entity for live tracking
                let tip_nav = nav_graph.add_node_with_entity(tip_world, NavNodeKind::Branch, tip);
                nav_graph.add_edge(current_nav_node, tip_nav);
                current_nav_node = tip_nav;
                world_pos = tip_world;
            }
            '+' => {
                pending_rotation =
                    pending_rotation * Quat::from_axis_angle(Vec3::X, lsystem.angle);
            }
            '-' => {
                pending_rotation =
                    pending_rotation * Quat::from_axis_angle(Vec3::X, -lsystem.angle);
            }
            '[' => {
                stack.push((
                    current_parent,
                    segment_length,
                    segment_radius,
                    pending_rotation,
                    world_pos,
                    world_rot,
                    current_nav_node,
                ));
                segment_length *= lsystem.shrink;
                segment_radius *= lsystem.radius_shrink;
                let twist = Quat::from_axis_angle(Vec3::Y, rng.random_range(0.5..2.5));
                pending_rotation = twist;
            }
            ']' => {
                // Spawn leaf at current tip
                let leaf_size = 2.0 * scale;
                let leaf = commands
                    .spawn((
                        Mesh3d(meshes.add(Sphere::new(leaf_size))),
                        MeshMaterial3d(leaf_material.clone()),
                        Transform::default(),
                    ))
                    .id();
                commands.entity(current_parent).add_child(leaf);

                if let Some((parent, len, rad, rot, wpos, wrot, nav)) = stack.pop() {
                    current_parent = parent;
                    segment_length = len;
                    segment_radius = rad;
                    pending_rotation = rot;
                    world_pos = wpos;
                    world_rot = wrot;
                    current_nav_node = nav;
                }
            }
            _ => {}
        }
    }
}
