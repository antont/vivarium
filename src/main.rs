use bevy::prelude::*;
use vivarium::components::*;
use vivarium::config::{Colors, Config};
use vivarium::lsystem_tree::spawn_tree;
use vivarium::nav_graph::NavGraph;
use vivarium::orbit_camera::{OrbitCamera, CameraMode, orbit_camera_system, camera_mode_system};
use vivarium::squirrel::{spawn_squirrel, squirrel_behavior_system, squirrel_movement_system};
use vivarium::spatial::SpatialIndex;
use vivarium::systems::boundary::boundary_force_system;
use vivarium::systems::face_velocity::face_velocity_system;
use vivarium::systems::brownian::brownian_motion_system;
use vivarium::systems::eating::eating_system;
use vivarium::systems::flocking::flocking_system;
use vivarium::systems::movement::movement_system;
use vivarium::systems::hunt::hunt_system;
use vivarium::systems::spatial_update::rebuild_spatial_index;
use vivarium::systems::swarm_cohesion::swarm_cohesion_system;
use vivarium::wind::{Wind, wind_update_system, wind_tree_system, setup_wind_indicator, wind_indicator_system};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Colors::BACKGROUND))
        .insert_resource(SpatialIndex::new(Config::SPATIAL_CELL_SIZE))
        .insert_resource(Wind::default())
        .insert_resource(CameraMode::default())
        .add_systems(Startup, (setup, setup_wind_indicator))
        .add_systems(
            Update,
            (
                wind_update_system,
                rebuild_spatial_index,
                (brownian_motion_system, swarm_cohesion_system, flocking_system),
                hunt_system,
                movement_system,
                wind_tree_system,
                face_velocity_system,
                eating_system,
                insect_respawn_system,
            )
                .chain(),
        )
        .add_systems(Update, (camera_mode_system, orbit_camera_system, wind_indicator_system, squirrel_behavior_system, squirrel_movement_system))
        .add_systems(PostUpdate, boundary_force_system)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Orbit camera
    commands.spawn((
        Camera3d::default(),
        Transform::default(),
        OrbitCamera::default(),
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

    // Ground plane at bottom of world
    let ground_size = Config::WORLD_HALF_SIZE * 2.0;
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(ground_size / 2.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Colors::GROUND,
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.0, -Config::WORLD_HALF_SIZE, 0.0)),
    ));

    // Build navigation graph and L-system trees
    let mut nav_graph = NavGraph::new();
    let ground_y = -Config::WORLD_HALF_SIZE;
    spawn_tree(&mut commands, &mut meshes, &mut materials, &mut nav_graph, Vec3::new(0.0, ground_y, 0.0), 1.0);
    spawn_tree(&mut commands, &mut meshes, &mut materials, &mut nav_graph, Vec3::new(-80.0, ground_y, 60.0), 0.8);
    spawn_tree(&mut commands, &mut meshes, &mut materials, &mut nav_graph, Vec3::new(50.0, ground_y, -70.0), 1.2);
    nav_graph.build_ground_nodes();

    // Spawn squirrels near tree bases, slightly above ground
    for i in 0..Config::SQUIRREL_COUNT {
        let x = [-10.0, -90.0, 40.0][i % 3];
        let z = [10.0, 70.0, -60.0][i % 3];
        spawn_squirrel(&mut commands, &mut meshes, &mut materials, Vec3::new(x, ground_y + 3.0, z), i);
    }

    commands.insert_resource(nav_graph);

    // Shared meshes and materials
    let insect_mesh = meshes.add(Sphere::new(Config::INSECT_RADIUS));
    let insect_material = materials.add(StandardMaterial {
        base_color: Colors::INSECT,
        unlit: true,
        ..default()
    });

    // Cone as bird body — tip points along +Y, face_velocity rotates it
    let bird_mesh = meshes.add(Cone {
        radius: Config::BIRD_RADIUS * 0.4,
        height: Config::BIRD_RADIUS * 2.5,
    });
    let bird_material = materials.add(StandardMaterial {
        base_color: Colors::BIRD,
        unlit: true,
        cull_mode: None, // visible from both sides
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
            SwarmCohesion {
                radius: Config::SWARM_COHESION_RADIUS,
                weight: Config::SWARM_COHESION_WEIGHT,
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
            HuntState::default(),
            Wander { strength: Config::BIRD_WANDER_STRENGTH },
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
            SwarmCohesion {
                radius: Config::SWARM_COHESION_RADIUS,
                weight: Config::SWARM_COHESION_WEIGHT,
            },
            BoundaryWrap,
            Mesh3d(mesh_handle.0.clone()),
            MeshMaterial3d(material_handle.0.clone()),
        ));
    }
}
