//! Visual demo: Bird nesting lifecycle
//!
//! A single tree with 2 birds. Birds are auto-fed on a timer so the
//! lifecycle progresses reliably without depending on actual hunting.
//! Watch birds fly to a branch, build a nest, incubate, and hatch.
//!
//! Run: cargo run --example nesting_demo
//!
//! Uses VivariumPlugin for all simulation systems — same behavior as main app.

use bevy::prelude::*;
use bevy::ecs::message::MessageWriter;
use vivarium::components::*;
use vivarium::config::{Colors, Config};
use vivarium::lsystem_tree::spawn_tree;
use vivarium::nav_graph::NavGraph;
use vivarium::orbit_camera::OrbitCamera;
use vivarium::VivariumPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VivariumPlugin)
        .insert_resource(ClearColor(Colors::BACKGROUND))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            auto_feed_system,
            status_text_system,
            log_state_system,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera — closer orbit
    commands.spawn((
        Camera3d::default(),
        Transform::default(),
        OrbitCamera {
            radius: 150.0,
            pitch: 0.6,
            ..default()
        },
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
    ));

    // Ground
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

    // One tree
    let mut nav_graph = NavGraph::new();
    let ground_y = -Config::WORLD_HALF_SIZE;
    spawn_tree(&mut commands, &mut meshes, &mut materials, &mut nav_graph, Vec3::new(0.0, ground_y, 0.0), 1.0);
    nav_graph.build_ground_nodes();
    commands.insert_resource(nav_graph);

    // Bird mesh + material
    let bird_mesh = meshes.add(Cone {
        radius: Config::BIRD_RADIUS * 0.4,
        height: Config::BIRD_RADIUS * 2.5,
    });
    let bird_material = materials.add(StandardMaterial {
        base_color: Colors::BIRD,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    // 2 birds
    commands.spawn((
        Bird,
        Transform::from_translation(Vec3::new(10.0, -10.0, 10.0)),
        Velocity(Vec3::new(1.0, 0.0, 0.0).normalize() * Config::BIRD_SPEED),
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
        BirdNestingState::default(),
        Wander { strength: Config::BIRD_WANDER_STRENGTH },
        BoundaryWrap,
        AutoFeedTimer(0.0),
        Mesh3d(bird_mesh.clone()),
        MeshMaterial3d(bird_material.clone()),
    ));

    commands.spawn((
        Bird,
        Transform::from_translation(Vec3::new(-20.0, 0.0, -15.0)),
        Velocity(Vec3::new(-1.0, 0.0, 1.0).normalize() * Config::BIRD_SPEED),
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
        BirdNestingState::default(),
        Wander { strength: Config::BIRD_WANDER_STRENGTH },
        BoundaryWrap,
        AutoFeedTimer(1.5),
        Mesh3d(bird_mesh.clone()),
        MeshMaterial3d(bird_material.clone()),
    ));

    // Status text
    commands.spawn((
        Text::new("Nesting Demo\n1: overview | Watch birds eat, nest, hatch"),
        TextFont { font_size: 18.0, ..default() },
        TextColor(Color::srgb(0.2, 0.2, 0.2)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        StatusText,
    ));

    info!("[nesting_demo] Setup complete: 2 birds (auto-fed), 1 tree");
}

/// Timer for auto-feeding birds in hunting phases.
#[derive(Component)]
struct AutoFeedTimer(f32);

/// Every 3 seconds, fire InsectEaten for birds that are in a hunting phase.
/// This bypasses the hunt/eat loop so the lifecycle progresses reliably.
const AUTO_FEED_INTERVAL: f32 = 3.0;

fn auto_feed_system(
    time: Res<Time>,
    mut birds: Query<(Entity, &BirdNestingState, &mut AutoFeedTimer), With<Bird>>,
    mut eaten_events: MessageWriter<InsectEaten>,
) {
    let dt = time.delta_secs();
    for (entity, nesting, mut timer) in &mut birds {
        let dominated = matches!(
            nesting.phase,
            BirdLifecycle::Hunting | BirdLifecycle::HuntingForEgg | BirdLifecycle::Parenting
        );
        if !dominated {
            continue;
        }
        timer.0 += dt;
        if timer.0 >= AUTO_FEED_INTERVAL {
            timer.0 = 0.0;
            info!("[nesting_demo] Auto-feeding bird {:?} (phase={:?})", entity, nesting.phase);
            eaten_events.write(InsectEaten { bird: entity });
        }
    }
}

#[derive(Component)]
struct StatusText;

/// Log lifecycle transitions periodically for observability.
fn log_state_system(
    birds: Query<(Entity, &BirdNestingState), With<Bird>>,
    nests: Query<&Nest>,
    hatchlings: Query<&Hatchling>,
    time: Res<Time>,
) {
    // Log every ~2 seconds
    let t = time.elapsed_secs();
    if (t * 10.0) as u32 % 20 != 0 {
        return;
    }

    for (entity, nesting) in &birds {
        let phase = match nesting.phase {
            BirdLifecycle::Hunting => "Hunting",
            BirdLifecycle::FlyingToNest => "FlyingToNest",
            BirdLifecycle::Building => "Building",
            BirdLifecycle::HuntingForEgg => "HuntingForEgg",
            BirdLifecycle::Incubating => "Incubating",
            BirdLifecycle::Parenting => "Parenting",
            BirdLifecycle::Defending => "Defending",
        };
        info!(
            "[nesting_demo] Bird {:?}: phase={}, eaten={}, timer={:.1}, nest={:?}",
            entity, phase, nesting.insects_eaten, nesting.timer, nesting.nest
        );
    }

    let nest_count = nests.iter().count();
    let hatchling_count = hatchlings.iter().count();
    if nest_count > 0 || hatchling_count > 0 {
        info!(
            "[nesting_demo] World: nests={}, hatchlings={}",
            nest_count, hatchling_count
        );
    }
}

/// Update status text with current state.
fn status_text_system(
    birds: Query<&BirdNestingState, With<Bird>>,
    nests: Query<&Nest>,
    hatchlings: Query<&Hatchling>,
    mut text_q: Query<&mut Text, With<StatusText>>,
) {
    let Ok(mut text) = text_q.single_mut() else { return };

    let mut lines = vec!["Nesting Demo (auto-fed)".to_string()];

    for (i, nesting) in birds.iter().enumerate() {
        let phase = match nesting.phase {
            BirdLifecycle::Hunting => "Hunting",
            BirdLifecycle::FlyingToNest => "Flying to nest",
            BirdLifecycle::Building => "Building nest",
            BirdLifecycle::HuntingForEgg => "Hunting for egg",
            BirdLifecycle::Incubating => "Incubating",
            BirdLifecycle::Parenting => "Parenting",
            BirdLifecycle::Defending => "Defending!",
        };
        lines.push(format!("Bird {}: {}", i + 1, phase));
    }

    lines.push(format!("Nests: {}  Hatchlings: {}", nests.iter().count(), hatchlings.iter().count()));

    **text = lines.join("\n");
}
