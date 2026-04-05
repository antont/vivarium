use bevy::prelude::*;
use rand::Rng;
use crate::components::{TreeAnchor, TreeSegment};

/// A simple L-system tree generator.
/// Rules: F → F[+F]F[-F]F  (branching pattern)
/// Symbols: F = forward, + = turn right, - = turn left, [ = push, ] = pop

struct Segment {
    start: Vec3,
    end: Vec3,
    radius: f32,
}

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

    fn interpret(&self, instructions: &str) -> (Vec<Segment>, Vec<Vec3>) {
        let mut segments = Vec::new();
        let mut leaves = Vec::new();
        let mut pos = Vec3::ZERO;
        let mut dir = Vec3::Y; // grow upward
        let mut right = Vec3::X;
        let mut stack: Vec<(Vec3, Vec3, Vec3, f32, f32)> = Vec::new();
        let mut length = self.length;
        let mut radius = self.radius;
        let mut rng = rand::rng();

        for ch in instructions.chars() {
            match ch {
                'F' => {
                    // Add slight random wobble
                    let wobble = Vec3::new(
                        rng.random_range(-0.1..0.1),
                        rng.random_range(-0.05..0.05),
                        rng.random_range(-0.1..0.1),
                    );
                    let actual_dir = (dir + wobble).normalize_or_zero();
                    let end = pos + actual_dir * length;
                    segments.push(Segment {
                        start: pos,
                        end,
                        radius,
                    });
                    pos = end;
                }
                '+' => {
                    // Rotate around a random-ish axis (not just one plane)
                    let axis = right;
                    let rot = Quat::from_axis_angle(axis, self.angle);
                    dir = rot * dir;
                    right = rot * right;
                }
                '-' => {
                    let axis = right;
                    let rot = Quat::from_axis_angle(axis, -self.angle);
                    dir = rot * dir;
                    right = rot * right;
                }
                '[' => {
                    stack.push((pos, dir, right, length, radius));
                    length *= self.shrink;
                    radius *= self.radius_shrink;
                    // Twist around the up axis for 3D spread
                    let twist = Quat::from_axis_angle(Vec3::Y, rng.random_range(0.5..2.5));
                    dir = twist * dir;
                    right = twist * right;
                }
                ']' => {
                    // Leaf at branch tip
                    leaves.push(pos);
                    if let Some((p, d, r, l, rad)) = stack.pop() {
                        pos = p;
                        dir = d;
                        right = r;
                        length = l;
                        radius = rad;
                    }
                }
                _ => {}
            }
        }

        (segments, leaves)
    }
}

/// Spawn a procedural L-system tree at the given position.
pub fn spawn_tree(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    scale: f32,
) {
    let lsystem = LSystem {
        axiom: "F".to_string(),
        iterations: 3,
        angle: 1.2, // ~26 degrees
        length: 10.0 * scale,
        shrink: 0.65,
        radius: 2.0 * scale,
        radius_shrink: 0.6,
    };

    let instructions = lsystem.generate();
    let (segments, leaves) = lsystem.interpret(&instructions);

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

    // Spawn each branch segment as a cylinder
    for seg in &segments {
        let mid = (seg.start + seg.end) / 2.0;
        let diff = seg.end - seg.start;
        let len = diff.length();
        if len < f32::EPSILON {
            continue;
        }
        let dir = diff / len;

        // Cylinder is centered at origin along Y axis
        let rotation = Quat::from_rotation_arc(Vec3::Y, dir);

        let world_pos = position + mid;
        let transform = Transform::from_translation(world_pos).with_rotation(rotation);
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(seg.radius, len))),
            MeshMaterial3d(bark_material.clone()),
            transform,
            TreeSegment,
            TreeAnchor {
                root: position,
                local_offset: mid,
                base_rotation: rotation,
            },
        ));
    }

    // Spawn leaf clusters as spheres at branch tips
    for &leaf_pos in &leaves {
        let leaf_size = 2.0 * scale;
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(leaf_size))),
            MeshMaterial3d(leaf_material.clone()),
            Transform::from_translation(position + leaf_pos),
            TreeSegment,
            TreeAnchor {
                root: position,
                local_offset: leaf_pos,
                base_rotation: Quat::IDENTITY,
            },
        ));
    }
}
