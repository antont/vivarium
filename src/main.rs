use bevy::prelude::*;
use vivarium::components::*;
use vivarium::config::{Colors, Config};
use vivarium::spatial::SpatialIndex;
use vivarium::systems::boundary::boundary_wrap_system;
use vivarium::systems::brownian::brownian_motion_system;
use vivarium::systems::eating::eating_system;
use vivarium::systems::flocking::flocking_system;
use vivarium::systems::movement::movement_system;
use vivarium::systems::predator::predator_sight_system;
use vivarium::systems::spatial_update::rebuild_spatial_index;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(SpatialIndex::new(Config::SPATIAL_CELL_SIZE))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                rebuild_spatial_index,
                (brownian_motion_system, flocking_system),
                predator_sight_system,
                movement_system,
                eating_system,
                insect_respawn_system,
            )
                .chain(),
        )
        .add_systems(PostUpdate, boundary_wrap_system)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 300.0, 400.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Lighting
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
    ));

    // Shared meshes and materials
    let insect_mesh = meshes.add(Sphere::new(Config::INSECT_RADIUS));
    let insect_material = materials.add(StandardMaterial {
        base_color: Colors::INSECT,
        unlit: true,
        ..default()
    });

    let bird_mesh = meshes.add(Sphere::new(Config::BIRD_RADIUS));
    let bird_material = materials.add(StandardMaterial {
        base_color: Colors::BIRD,
        unlit: true,
        ..default()
    });

    // Store handles as resources for respawning
    commands.insert_resource(InsectMeshHandle(insect_mesh.clone()));
    commands.insert_resource(InsectMaterialHandle(insect_material.clone()));

    // Spawn insects with meshes
    let mut rng = rand::rng();
    let half = Config::WORLD_HALF_SIZE;

    for _ in 0..Config::INSECT_COUNT {
        let position = Vec3::new(
            rng.random_range(-half..half),
            rng.random_range(-half..half),
            rng.random_range(-half..half),
        );
        let direction = random_direction(&mut rng);

        commands.spawn((
            Insect,
            Transform::from_translation(position),
            Velocity(direction * Config::INSECT_SPEED),
            BrownianMotion {
                wander_strength: Config::INSECT_WANDER_STRENGTH,
            },
            BoundaryWrap,
            Mesh3d(insect_mesh.clone()),
            MeshMaterial3d(insect_material.clone()),
        ));
    }

    // Spawn birds with meshes
    for _ in 0..Config::BIRD_COUNT {
        let position = Vec3::new(
            rng.random_range(-half * 0.3..half * 0.3),
            rng.random_range(-half * 0.3..half * 0.3),
            rng.random_range(-half * 0.3..half * 0.3),
        );
        let direction = random_direction(&mut rng);

        commands.spawn((
            Bird,
            Transform::from_translation(position),
            Velocity(direction * Config::BIRD_SPEED),
            Predator {
                sight_range: Config::BIRD_SIGHT_RANGE,
                sight_half_angle: Config::BIRD_SIGHT_HALF_ANGLE,
            },
            Flocking {
                separation_weight: Config::SEPARATION_WEIGHT,
                alignment_weight: Config::ALIGNMENT_WEIGHT,
                cohesion_weight: Config::COHESION_WEIGHT,
            },
            BoundaryWrap,
            Mesh3d(bird_mesh.clone()),
            MeshMaterial3d(bird_material.clone()),
        ));
    }
}

fn random_direction(rng: &mut impl Rng) -> Vec3 {
    Vec3::new(
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
    )
    .normalize_or_zero()
}

/// Resource handles for respawning insects with meshes.
#[derive(Resource)]
struct InsectMeshHandle(Handle<Mesh>);

#[derive(Resource)]
struct InsectMaterialHandle(Handle<StandardMaterial>);

fn insect_respawn_system(
    mut commands: Commands,
    insects: Query<&Insect>,
    mesh_handle: Option<Res<InsectMeshHandle>>,
    material_handle: Option<Res<InsectMaterialHandle>>,
) {
    let count = insects.iter().count();
    if count >= Config::MIN_INSECT_COUNT {
        return;
    }

    let (Some(mesh_handle), Some(material_handle)) = (mesh_handle, material_handle) else {
        return;
    };

    let mut rng = rand::rng();
    let half = Config::WORLD_HALF_SIZE;

    for _ in 0..Config::INSECT_RESPAWN_BATCH {
        let position = Vec3::new(
            rng.random_range(-half..half),
            rng.random_range(-half..half),
            rng.random_range(-half..half),
        );
        let direction = random_direction(&mut rng);

        commands.spawn((
            Insect,
            Transform::from_translation(position),
            Velocity(direction * Config::INSECT_SPEED),
            BrownianMotion {
                wander_strength: Config::INSECT_WANDER_STRENGTH,
            },
            BoundaryWrap,
            Mesh3d(mesh_handle.0.clone()),
            MeshMaterial3d(material_handle.0.clone()),
        ));
    }
}
