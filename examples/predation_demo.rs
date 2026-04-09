//! Visual demo: Squirrel-bird predation
//!
//! A tree with a bird that already has a hatchling, and a squirrel nearby.
//! Watch the squirrel detect the hatchling, approach it, the hatchling alert,
//! and the parent bird rush to defend.
//!
//! Run: cargo run --example predation_demo
//!
//! Uses VivariumPlugin for all simulation systems — same behavior as main app.

use bevy::prelude::*;
use vivarium::components::*;
use vivarium::config::{Colors, Config};
use vivarium::lsystem_tree::spawn_tree;
use vivarium::nav_graph::NavGraph;
use vivarium::orbit_camera::OrbitCamera;
use vivarium::squirrel::spawn_squirrel;
use vivarium::VivariumPlugin;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VivariumPlugin)
        .insert_resource(ClearColor(Colors::BACKGROUND))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            insect_respawn,
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
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::default(),
        OrbitCamera {
            radius: 120.0,
            pitch: 0.5,
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

    // Find a branch node for the nest
    let branch_node = nav_graph.nodes.iter().enumerate()
        .find(|(_, n)| n.kind == vivarium::nav_graph::NavNodeKind::Branch)
        .map(|(i, _)| i)
        .expect("Should have branch nodes");
    let nest_pos = nav_graph.nodes[branch_node].position;

    // Parent bird — starts in Parenting phase, flying nearby
    let bird = commands.spawn((
        Bird,
        Transform::from_translation(nest_pos + Vec3::new(40.0, 20.0, 0.0)),
        Velocity(Vec3::new(-1.0, 0.0, 0.0).normalize() * Config::BIRD_SPEED),
        Predator {
            sight_range: Config::BIRD_SIGHT_RANGE,
            sight_half_angle: Config::BIRD_SIGHT_HALF_ANGLE,
        },
        Flocking {
            separation_weight: 0.0,
            alignment_weight: 0.0,
            cohesion_weight: 0.0,
        },
        HuntState::default(),
        BirdNestingState {
            phase: BirdLifecycle::Parenting,
            nest: None, // will be set below
            nest_nav_node: Some(branch_node),
            timer: 0.0,
            insects_eaten: 0,
        },
        Wander { strength: Config::BIRD_WANDER_STRENGTH },
        BoundaryWrap,
        Visibility::default(),
    )).id();

    // Nest on the branch — parented to branch entity so it moves with wind
    let branch_entity = nav_graph.nodes[branch_node].entity;
    let nest = commands.spawn((
        Nest {
            parent_bird: bird,
            nav_node: branch_node,
            has_egg: false,
            has_hatchling: true,
        },
        Transform::default(),
    )).id();
    if let Some(parent) = branch_entity {
        commands.entity(parent).add_child(nest);
    } else {
        commands.entity(nest).insert(Transform::from_translation(nest_pos));
    }

    // Hatchling as child of nest
    let hatchling = commands.spawn((
        Hatchling {
            nest,
            parent_bird: bird,
            alert: false,
        },
        Transform::from_translation(Vec3::new(0.0, 1.5, 0.0)),
    )).id();
    commands.entity(nest).add_child(hatchling);

    // Fix bird's nest reference
    commands.entity(bird).insert(BirdNestingState {
        phase: BirdLifecycle::Parenting,
        nest: Some(nest),
        nest_nav_node: Some(branch_node),
        timer: 0.0,
        insects_eaten: 0,
    });

    info!("[predation_demo] Bird {:?} parenting at nest {:?} (branch node {})", bird, nest, branch_node);

    // Squirrel on the ground near the tree
    spawn_squirrel(&mut commands, Vec3::new(20.0, ground_y + 3.0, 20.0), 0);

    commands.insert_resource(nav_graph);

    // Insects for the bird to hunt — enough to reliably find prey
    let mut rng = rand::rng();
    for _ in 0..20 {
        let pos = Vec3::new(
            rng.random_range(-40.0..40.0),
            rng.random_range(-20.0..20.0),
            rng.random_range(-40.0..40.0),
        );
        let dir = Vec3::new(
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
        ).normalize_or_zero();
        commands.spawn((
            Insect,
            Transform::from_translation(pos),
            Velocity(dir * Config::INSECT_SPEED),
            BrownianMotion { wander_strength: Config::INSECT_WANDER_STRENGTH },
            SwarmCohesion {
                radius: Config::SWARM_COHESION_RADIUS,
                weight: Config::SWARM_COHESION_WEIGHT,
            },
            BoundaryWrap,
            Visibility::default(),
        ));
    }

    // Status text
    commands.spawn((
        Text::new("Predation Demo"),
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

    info!("[predation_demo] Setup complete: 1 bird (parenting), 1 squirrel, 1 nest+hatchling, 20 insects");
}

#[derive(Component)]
struct StatusText;

fn insect_respawn(
    mut commands: Commands,
    insects: Query<&Insect>,
) {
    let count = insects.iter().count();
    if count >= 10 { return; }
    let mut rng = rand::rng();
    for _ in 0..5 {
        let pos = Vec3::new(
            rng.random_range(-40.0..40.0),
            rng.random_range(-20.0..20.0),
            rng.random_range(-40.0..40.0),
        );
        let dir = Vec3::new(
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
        ).normalize_or_zero();
        commands.spawn((
            Insect,
            Transform::from_translation(pos),
            Velocity(dir * Config::INSECT_SPEED),
            BrownianMotion { wander_strength: Config::INSECT_WANDER_STRENGTH },
            SwarmCohesion {
                radius: Config::SWARM_COHESION_RADIUS,
                weight: Config::SWARM_COHESION_WEIGHT,
            },
            BoundaryWrap,
            Visibility::default(),
        ));
    }
}

/// Log lifecycle transitions periodically for observability.
fn log_state_system(
    birds: Query<(Entity, &BirdNestingState), With<Bird>>,
    squirrels: Query<(Entity, &SquirrelState, Option<&SquirrelTarget>), With<Squirrel>>,
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
            BirdLifecycle::Defending => "DEFENDING",
        };
        info!(
            "[predation_demo] Bird {:?}: phase={}, eaten={}, timer={:.1}",
            entity, phase, nesting.insects_eaten, nesting.timer
        );
    }

    for (entity, state, target) in &squirrels {
        let phase = match state.phase {
            SquirrelPhase::Idle => "Idle",
            SquirrelPhase::Moving => "Moving",
            SquirrelPhase::Hunting => "HUNTING",
            SquirrelPhase::Fleeing => "FLEEING",
        };
        let target_info = if let Some(t) = target {
            format!(" -> hatchling {:?}", t.hatchling)
        } else {
            String::new()
        };
        info!(
            "[predation_demo] Squirrel {:?}: phase={}{}",
            entity, phase, target_info
        );
    }

    for hatchling in &hatchlings {
        if hatchling.alert {
            info!("[predation_demo] Hatchling ALERTING! parent={:?}", hatchling.parent_bird);
        }
    }
}

fn status_text_system(
    birds: Query<&BirdNestingState, With<Bird>>,
    hatchlings: Query<&Hatchling>,
    squirrels: Query<(&SquirrelState, Option<&SquirrelTarget>), With<Squirrel>>,
    mut text_q: Query<&mut Text, With<StatusText>>,
) {
    let Ok(mut text) = text_q.single_mut() else { return };

    let mut lines = vec!["Predation Demo".to_string()];

    for (i, nesting) in birds.iter().enumerate() {
        let phase = match nesting.phase {
            BirdLifecycle::Hunting => "Hunting",
            BirdLifecycle::FlyingToNest => "Flying to nest",
            BirdLifecycle::Building => "Building nest",
            BirdLifecycle::HuntingForEgg => "Hunting for egg",
            BirdLifecycle::Incubating => "Incubating",
            BirdLifecycle::Parenting => "Parenting",
            BirdLifecycle::Defending => "DEFENDING!",
        };
        lines.push(format!("Bird {}: {}", i + 1, phase));
    }

    for (i, (state, target)) in squirrels.iter().enumerate() {
        let phase = match state.phase {
            SquirrelPhase::Idle => "Idle",
            SquirrelPhase::Moving => "Wandering",
            SquirrelPhase::Hunting => "HUNTING hatchling!",
            SquirrelPhase::Fleeing => "FLEEING!",
        };
        let targeting = if target.is_some() { " [target locked]" } else { "" };
        lines.push(format!("Squirrel {}: {}{}", i + 1, phase, targeting));
    }

    for hatchling in &hatchlings {
        let alert = if hatchling.alert { " ALERTING!" } else { "" };
        lines.push(format!("Hatchling{}", alert));
    }

    **text = lines.join("\n");
}
