use bevy::prelude::*;
use crate::components::*;
use crate::config::Config;

/// Add visual mesh directly to birds that don't have one yet.
pub fn bird_visual_system(
    mut commands: Commands,
    birds: Query<Entity, (With<Bird>, Without<BirdVisual>)>,
    shared: Res<SharedMeshes>,
) {
    for entity in &birds {
        commands.entity(entity).insert((
            BirdVisual,
            Mesh3d(shared.bird_mesh.clone()),
            MeshMaterial3d(shared.bird_material.clone()),
        ));
    }
}

/// Add 3D asterisk visual to insects — three thin crossed bars.
pub fn insect_visual_system(
    mut commands: Commands,
    insects: Query<Entity, (With<Insect>, Without<InsectVisual>)>,
    shared: Res<SharedMeshes>,
) {
    let a = std::f32::consts::FRAC_PI_4;
    let axes = [
        Quat::IDENTITY,
        Quat::from_rotation_x(a),
        Quat::from_rotation_x(-a),
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        Quat::from_rotation_z(a) * Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        Quat::from_rotation_z(-a) * Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
    ];

    for entity in &insects {
        for &rot in &axes {
            let bar = commands.spawn((
                Mesh3d(shared.insect_bar_mesh.clone()),
                MeshMaterial3d(shared.insect_material.clone()),
                Transform::from_rotation(rot),
            )).id();
            commands.entity(entity).add_child(bar);
        }

        commands.entity(entity).insert(InsectVisual);
    }
}

/// Spawn visual meshes for squirrels that don't have one yet.
/// Reproduces the multi-part body hierarchy (body, head, ears, tail, feet).
pub fn squirrel_visual_system(
    mut commands: Commands,
    squirrels: Query<Entity, (With<Squirrel>, Without<SquirrelVisual>)>,
    shared: Res<SharedMeshes>,
) {
    for entity in &squirrels {
        let s = Config::SQUIRREL_BODY_SCALE;

        // Body — stretched sphere
        let body = commands.spawn((
            Mesh3d(shared.squirrel_body.clone()),
            MeshMaterial3d(shared.squirrel_fur.clone()),
            Transform::from_scale(Vec3::new(0.7, 0.5, 1.0)),
        )).id();
        commands.entity(entity).add_child(body);

        // Head — smaller sphere, forward and up
        let head = commands.spawn((
            Mesh3d(shared.squirrel_head.clone()),
            MeshMaterial3d(shared.squirrel_fur.clone()),
            Transform::from_translation(Vec3::new(0.0, s * 0.25, s * 0.6)),
        )).id();
        commands.entity(entity).add_child(head);

        // Ears — two tiny spheres
        for side in [-1.0_f32, 1.0] {
            let ear = commands.spawn((
                Mesh3d(shared.squirrel_ear.clone()),
                MeshMaterial3d(shared.squirrel_dark.clone()),
                Transform::from_translation(Vec3::new(
                    side * s * 0.2,
                    s * 0.45,
                    s * 0.55,
                )),
            )).id();
            commands.entity(entity).add_child(ear);
        }

        // Tail — pivot at body attachment, then offset mesh
        let tail_pivot = commands.spawn(
            Transform::from_translation(Vec3::new(0.0, s * 0.1, -s * 0.3))
                .with_rotation(Quat::from_axis_angle(Vec3::X, -0.8))
        ).id();
        commands.entity(entity).add_child(tail_pivot);

        let tail_mesh = commands.spawn((
            Mesh3d(shared.squirrel_tail.clone()),
            MeshMaterial3d(shared.squirrel_fur.clone()),
            Transform::from_translation(Vec3::new(0.0, s * 0.4, 0.0))
                .with_scale(Vec3::new(0.4, 1.2, 0.5)),
        )).id();
        commands.entity(tail_pivot).add_child(tail_mesh);

        // Feet — 4 tiny spheres
        for &(x, z) in &[(-0.3, 0.3), (0.3, 0.3), (-0.3, -0.2), (0.3, -0.2)] {
            let foot = commands.spawn((
                Mesh3d(shared.squirrel_foot.clone()),
                MeshMaterial3d(shared.squirrel_dark.clone()),
                Transform::from_translation(Vec3::new(x * s, -s * 0.35, z * s)),
            )).id();
            commands.entity(entity).add_child(foot);
        }

        commands.entity(entity).insert(SquirrelVisual);
    }
}
