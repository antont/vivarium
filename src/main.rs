use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use vivarium::components::*;
use vivarium::config::{Colors, Config};
use vivarium::lsystem_tree::spawn_tree;
use vivarium::nav_graph::NavGraph;
use vivarium::orbit_camera::OrbitCamera;
use vivarium::wind::setup_wind_indicator;
use vivarium::VivariumPlugin;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            bevy::diagnostic::EntityCountDiagnosticsPlugin::default(),
            bevy::diagnostic::LogDiagnosticsPlugin::default(),
            bevy::render::diagnostic::RenderDiagnosticsPlugin,
        ))
        .add_plugins(VivariumPlugin)
        .insert_resource(ClearColor(Colors::BACKGROUND))
        .add_systems(Startup, (setup, setup_wind_indicator, setup_status_ui))
        .add_systems(Update, (
            vivarium::wind::wind_indicator_system,
            insect_respawn_system,
            status_ui_system,
            periodic_log_system,
        ))
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
        Tonemapping::None,
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
        vivarium::squirrel::spawn_squirrel(&mut commands, Vec3::new(x, ground_y + 3.0, z), i);
    }

    commands.insert_resource(nav_graph);

    // Spawn insects (logic only — visuals added by insect_visual_system)
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
            BrownianMotion { wander_strength: Config::INSECT_WANDER_STRENGTH },
            SwarmCohesion {
                radius: Config::SWARM_COHESION_RADIUS,
                weight: Config::SWARM_COHESION_WEIGHT,
            },
            BoundaryWrap,
            Visibility::default(),
        ));
    }

    // Spawn birds (logic only — visuals added by bird_visual_system)
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
            BirdNestingState::default(),
            Wander { strength: Config::BIRD_WANDER_STRENGTH },
            BoundaryWrap,
            Visibility::default(),
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

#[derive(Component)]
struct StatusText;

fn setup_status_ui(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgb(0.15, 0.15, 0.15)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        StatusText,
    ));
}

fn status_ui_system(
    birds: Query<&BirdNestingState, With<Bird>>,
    insects: Query<&Insect>,
    nests: Query<&Nest>,
    hatchlings: Query<&Hatchling>,
    squirrels: Query<(&SquirrelState, Option<&SquirrelTarget>), With<Squirrel>>,
    mut text_q: Query<&mut Text, With<StatusText>>,
) {
    let Ok(mut text) = text_q.single_mut() else { return };

    let mut phase_counts = [0u32; 7];
    for nesting in &birds {
        let idx = match nesting.phase {
            BirdLifecycle::Hunting => 0,
            BirdLifecycle::FlyingToNest => 1,
            BirdLifecycle::Building => 2,
            BirdLifecycle::HuntingForEgg => 3,
            BirdLifecycle::Incubating => 4,
            BirdLifecycle::Parenting => 5,
            BirdLifecycle::Defending => 6,
        };
        phase_counts[idx] += 1;
    }

    let mut sq_phases = [0u32; 4];
    let mut sq_targeting = 0u32;
    for (state, target) in &squirrels {
        let idx = match state.phase {
            SquirrelPhase::Idle => 0,
            SquirrelPhase::Moving => 1,
            SquirrelPhase::Hunting => 2,
            SquirrelPhase::Fleeing => 3,
        };
        sq_phases[idx] += 1;
        if target.is_some() { sq_targeting += 1; }
    }

    let mut lines = Vec::new();
    lines.push(format!(
        "Insects: {}  Birds: {}  Nests: {}  Hatchlings: {}",
        insects.iter().count(), birds.iter().count(),
        nests.iter().count(), hatchlings.iter().count(),
    ));

    let labels = ["Hunt", "FlyToNest", "Build", "HuntEgg", "Incubate", "Parent", "Defend"];
    let bird_parts: Vec<String> = labels.iter().zip(phase_counts.iter())
        .filter(|(_, c)| **c > 0)
        .map(|(l, c)| format!("{}:{}", l, c))
        .collect();
    if !bird_parts.is_empty() {
        lines.push(format!("Birds: {}", bird_parts.join(" ")));
    }

    let sq_labels = ["Idle", "Moving", "Hunting", "Fleeing"];
    let sq_parts: Vec<String> = sq_labels.iter().zip(sq_phases.iter())
        .filter(|(_, c)| **c > 0)
        .map(|(l, c)| format!("{}:{}", l, c))
        .collect();
    if !sq_parts.is_empty() {
        lines.push(format!("Squirrels: {}  targeting:{}", sq_parts.join(" "), sq_targeting));
    }

    **text = lines.join("\n");
}

#[derive(Resource, Default)]
struct LastLogTime(f32);

fn periodic_log_system(
    birds: Query<&BirdNestingState, With<Bird>>,
    insects: Query<&Insect>,
    nests: Query<&Nest>,
    hatchlings: Query<&Hatchling>,
    squirrels: Query<(&SquirrelState, Option<&SquirrelTarget>), With<Squirrel>>,
    time: Res<Time>,
    mut last_log: Local<LastLogTime>,
) {
    let t = time.elapsed_secs();
    if t - last_log.0 < 5.0 {
        return;
    }
    last_log.0 = t;

    let mut phase_counts = [0u32; 7];
    for nesting in &birds {
        let idx = match nesting.phase {
            BirdLifecycle::Hunting => 0,
            BirdLifecycle::FlyingToNest => 1,
            BirdLifecycle::Building => 2,
            BirdLifecycle::HuntingForEgg => 3,
            BirdLifecycle::Incubating => 4,
            BirdLifecycle::Parenting => 5,
            BirdLifecycle::Defending => 6,
        };
        phase_counts[idx] += 1;
    }

    let labels = ["Hunt", "FlyToNest", "Build", "HuntEgg", "Incubate", "Parent", "Defend"];
    let bird_summary: Vec<String> = labels.iter().zip(phase_counts.iter())
        .filter(|(_, c)| **c > 0)
        .map(|(l, c)| format!("{}:{}", l, c))
        .collect();

    let mut sq_hunting = 0;
    let mut sq_fleeing = 0;
    for (state, _) in &squirrels {
        match state.phase {
            SquirrelPhase::Hunting => sq_hunting += 1,
            SquirrelPhase::Fleeing => sq_fleeing += 1,
            _ => {}
        }
    }

    info!(
        "[sim] t={:.0}s insects={} birds=[{}] nests={} hatchlings={} sq_hunt={} sq_flee={}",
        t,
        insects.iter().count(),
        bird_summary.join(" "),
        nests.iter().count(),
        hatchlings.iter().count(),
        sq_hunting,
        sq_fleeing,
    );
}

fn insect_respawn_system(
    mut commands: Commands,
    insects: Query<&Insect>,
) {
    let count = insects.iter().count();
    if count >= Config::MIN_INSECT_COUNT {
        return;
    }

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
