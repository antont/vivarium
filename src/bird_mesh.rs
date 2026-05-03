use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_asset::RenderAssetUsages;
use bevy_mesh::Indices;

/// Build a simple bird mesh: elongated body + swept wings + pointed beak + tail fan.
/// The bird points along +Y (head/beak at top, tail at bottom).
/// Returns a Mesh with ~70 vertices and ~110 triangles.
pub fn create_bird_mesh(scale: f32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);

    // ---- Body ----
    // Stack of rings along Y (head at +Y, tail at -Y)
    // 6 rings x 8 vertices each = 48 body vertices
    let rings = [
        // (y, radius) — from head to tail
        (1.8, 0.0),  // 0: beak tip (single point)
        (1.4, 0.25), // 1: narrow head
        (0.9, 0.45), // 2: neck/shoulder
        (0.3, 0.5),  // 3: mid-body (widest)
        (-0.4, 0.4), // 4: lower body
        (-1.0, 0.2), // 5: tail base (narrows)
    ];
    let segments = 8; // vertices per ring

    // Body vertex indices: ring r, segment s => index r * segments + s
    // Build indices for body tubes between consecutive rings
    let mut indices: Vec<u32> = Vec::new();

    for r in 0..(rings.len() - 1) {
        let (_y_a, rad_a) = rings[r];
        let (_y_b, rad_b) = rings[r + 1];

        for s in 0..segments {
            let s_next = (s + 1) % segments;
            let i0 = (r * segments + s) as u32;
            let i1 = (r * segments + s_next) as u32;
            let i2 = ((r + 1) * segments + s) as u32;
            let i3 = ((r + 1) * segments + s_next) as u32;

            if rad_a > 0.001 {
                indices.push(i0);
                indices.push(i2);
                indices.push(i1);
            }
            if rad_b > 0.001 {
                indices.push(i1);
                indices.push(i2);
                indices.push(i3);
            }
        }
    }

    // ---- Wings ----
    // Each wing: 2 triangles forming a swept-back shape
    // Wing root on body, tip out and slightly back
    // 8 wing vertices (4 per wing) => indices start after body
    let body_verts = rings.len() * segments;
    let wing_base = body_verts;

    // Left wing: root near mid-body, tip extends +X and sweeps toward -Y
    // Right wing: mirror along -X
    let wing_data = [
        // Left wing  (vertices 0..3 within wing block)
        // Root front
        (0.45, 0.6, 0.3),
        // Root back
        (0.45, 0.6, -0.5),
        // Tip front  (extends outward, sweeps back)
        (3.5, 0.35, 0.1),
        // Tip back
        (3.5, 0.35, -0.9),
        // Right wing (mirror: negate X)
        // Root front
        (-0.45, 0.6, 0.3),
        // Root back
        (-0.45, 0.6, -0.5),
        // Tip front
        (-3.5, 0.35, 0.1),
        // Tip back
        (-3.5, 0.35, -0.9),
    ];

    // Wing indices (each wing = 2 triangles = 6 indices)
    // Left wing
    indices.push(wing_base as u32);
    indices.push((wing_base + 2) as u32);
    indices.push((wing_base + 1) as u32);
    indices.push((wing_base + 1) as u32);
    indices.push((wing_base + 2) as u32);
    indices.push((wing_base + 3) as u32);
    // Right wing (winding order flipped for correct normals)
    indices.push((wing_base + 4) as u32);
    indices.push((wing_base + 5) as u32);
    indices.push((wing_base + 6) as u32);
    indices.push((wing_base + 5) as u32);
    indices.push((wing_base + 7) as u32);
    indices.push((wing_base + 6) as u32);

    // ---- Tail fan ----
    // 4 triangles fanning out at the tail end
    let tail_base = body_verts + 8;
    let tail_data = [
        // Center (at tail base)
        (0.0, -1.0, 0.0),
        // Left-up
        (-0.7, -1.4, -0.1),
        // Right-up
        (0.7, -1.4, -0.1),
        // Left-down
        (-0.5, -1.5, 0.0),
        // Right-down
        (0.5, -1.5, 0.0),
    ];

    // Tail triangles: center to each adjacent pair
    indices.push(tail_base as u32);
    indices.push((tail_base + 1) as u32);
    indices.push((tail_base + 2) as u32);
    indices.push(tail_base as u32);
    indices.push((tail_base + 2) as u32);
    indices.push((tail_base + 4) as u32);
    indices.push(tail_base as u32);
    indices.push((tail_base + 4) as u32);
    indices.push((tail_base + 3) as u32);
    indices.push(tail_base as u32);
    indices.push((tail_base + 3) as u32);
    indices.push((tail_base + 1) as u32);

    // ---- Assemble all vertices ----
    let total_verts = body_verts + 8 + 5; // body + wings + tail
    let mut positions: Vec<Vec3> = Vec::with_capacity(total_verts);
    let mut normals: Vec<Vec3> = Vec::with_capacity(total_verts);

    // Body vertices
    for (_r, (y, rad)) in rings.iter().enumerate() {
        for s in 0..segments {
            let angle = (s as f32 / segments as f32) * std::f32::consts::TAU;
            let cx = rad * angle.cos();
            let cz = rad * angle.sin();
            positions.push(Vec3::new(cx * scale, y * scale, cz * scale));

            // Normal: approximate as outward from local Y-axis cylinder
            if *rad > 0.001 {
                let n = Vec3::new(cx, 0.0, cz).normalize_or_zero();
                normals.push(n);
            } else {
                // Tip: point along +Y
                normals.push(Vec3::Y);
            }
        }
    }

    // Wing vertices
    for &(wx, wy, wz) in &wing_data {
        positions.push(Vec3::new(wx * scale, wy * scale, wz * scale));
        // Wing normals: point slightly up (+Y) for visible top surface
        normals.push(Vec3::new(0.0, 0.8, -0.2).normalize_or_zero());
    }

    // Tail vertices
    for &(tx, ty, tz) in &tail_data {
        positions.push(Vec3::new(tx * scale, ty * scale, tz * scale));
        normals.push(Vec3::new(0.0, -0.5, -0.3).normalize_or_zero());
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}
