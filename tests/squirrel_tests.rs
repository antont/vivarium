use bevy::prelude::*;
use vivarium::squirrel::project_to_cylinder;

/// The core bug: when a point is on or near the cylinder axis,
/// the radial direction is undefined and flips randomly, causing
/// the projected surface point to jump 180° every frame.
#[test]
fn projection_on_axis_should_not_flip() {
    let axis_start = Vec3::new(0.0, 0.0, 0.0);
    let axis_end = Vec3::new(0.0, 10.0, 0.0);
    let radius = 2.0;

    // Point exactly on the axis (this is where nav path nodes are)
    let on_axis = Vec3::new(0.0, 5.0, 0.0);
    let (pos1, norm1) = project_to_cylinder(on_axis, axis_start, axis_end, radius, None);

    // Point barely off axis in different directions — simulates floating point noise
    let slightly_positive = Vec3::new(0.001, 5.0, 0.0);
    let slightly_negative = Vec3::new(-0.001, 5.0, 0.0);

    let (pos2, norm2) = project_to_cylinder(slightly_positive, axis_start, axis_end, radius, None);
    let (pos3, norm3) = project_to_cylinder(slightly_negative, axis_start, axis_end, radius, None);

    // The normals should NOT be opposite directions — that's the flicker
    let dot = norm2.dot(norm3);
    assert!(
        dot > 0.0,
        "Near-axis points should project to the same side, not opposite. dot={}, n2={:?}, n3={:?}",
        dot, norm2, norm3
    );

    // All three projections should be at the cylinder surface (distance = radius from axis)
    let dist1 = Vec3::new(pos1.x, 0.0, pos1.z).length();
    let dist2 = Vec3::new(pos2.x, 0.0, pos2.z).length();
    let dist3 = Vec3::new(pos3.x, 0.0, pos3.z).length();
    assert!((dist1 - radius).abs() < 0.01, "pos1 should be on surface: dist={}", dist1);
    assert!((dist2 - radius).abs() < 0.01, "pos2 should be on surface: dist={}", dist2);
    assert!((dist3 - radius).abs() < 0.01, "pos3 should be on surface: dist={}", dist3);
}

/// Simulates sequential frames of a squirrel moving along a branch.
/// The projected positions should be smooth — no large jumps.
#[test]
fn sequential_projections_should_be_smooth() {
    let axis_start = Vec3::new(0.0, 0.0, 0.0);
    let axis_end = Vec3::new(0.0, 20.0, 0.0);
    let radius = 2.0;

    let mut last_pos: Option<Vec3> = None;
    let mut last_normal: Option<Vec3> = None;

    // Simulate 60 frames of movement along the axis (through the center)
    for i in 0..60 {
        let t = i as f32 / 60.0;
        let center = axis_start.lerp(axis_end, t);
        // Add tiny noise like floating point jitter
        let noisy = center + Vec3::new(
            (i as f32 * 0.7).sin() * 0.0001,
            0.0,
            (i as f32 * 1.3).cos() * 0.0001,
        );

        let (pos, normal) = project_to_cylinder(noisy, axis_start, axis_end, radius, last_normal);

        if let Some(prev) = last_pos {
            let jump = pos.distance(prev);
            assert!(
                jump < radius, // max reasonable frame-to-frame distance
                "Frame {}: position jumped {} units (prev={:?}, curr={:?}). Should be smooth.",
                i, jump, prev, pos
            );
        }
        last_pos = Some(pos);
        last_normal = Some(normal);
    }
}
