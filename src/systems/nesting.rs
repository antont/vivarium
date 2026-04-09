use bevy::prelude::*;
use bevy::ecs::message::MessageReader;
use rand::Rng;
use crate::components::*;
use crate::config::{Colors, Config};
use crate::nav_graph::{NavGraph, NavNodeKind};

/// Spawn visual meshes for nests that don't have one yet.
pub fn nest_visual_system(
    mut commands: Commands,
    nests: Query<Entity, (With<Nest>, Without<NestVisual>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in &nests {
        let mesh = commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(3.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Colors::NEST,
                ..default()
            })),
            Transform::default(),
        )).id();
        commands.entity(entity).insert(NestVisual).add_child(mesh);
    }
}

/// Spawn visual meshes for hatchlings that don't have one yet.
pub fn hatchling_visual_system(
    mut commands: Commands,
    hatchlings: Query<Entity, (With<Hatchling>, Without<HatchlingVisual>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in &hatchlings {
        let mesh = commands.spawn((
            Mesh3d(meshes.add(Sphere::new(1.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Colors::HATCHLING,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(Vec3::new(0.0, 1.5, 0.0)),
        )).id();
        commands.entity(entity).insert(HatchlingVisual).add_child(mesh);
    }
}

/// Bird lifecycle state machine — reacts to InsectEaten events and manages transitions.
pub fn bird_lifecycle_system(
    mut commands: Commands,
    time: Res<Time>,
    nav: Res<NavGraph>,
    mut events: MessageReader<InsectEaten>,
    mut birds: Query<(Entity, &Transform, &mut BirdNestingState, &mut Velocity), With<Bird>>,
    nests: Query<&Nest>,
    global_transforms: Query<&GlobalTransform>,
) {
    let dt = time.delta_secs();

    // Collect InsectEaten events by bird entity
    let mut eaten_birds: Vec<Entity> = Vec::new();
    for ev in events.read() {
        eaten_birds.push(ev.bird);
    }

    for (bird_entity, transform, mut nesting, mut velocity) in &mut birds {
        let ate_insect = eaten_birds.contains(&bird_entity);
        let prev_phase = nesting.phase;

        match nesting.phase {
            BirdLifecycle::Hunting => {
                if ate_insect {
                    // Pick an unoccupied branch node for nesting
                    let occupied: Vec<usize> = nests.iter().map(|n| n.nav_node).collect();
                    let branch_nodes: Vec<usize> = nav.nodes.iter().enumerate()
                        .filter(|(i, n)| n.kind == NavNodeKind::Branch && !occupied.contains(i))
                        .map(|(i, _)| i)
                        .collect();

                    if !branch_nodes.is_empty() {
                        let mut rng = rand::rng();
                        let node = branch_nodes[rng.random_range(0..branch_nodes.len())];
                        nesting.nest_nav_node = Some(node);
                        nesting.phase = BirdLifecycle::FlyingToNest;
                    } else {
                        info!("[nesting] Bird {:?} ate insect but no free branch nodes ({} occupied)", bird_entity, occupied.len());
                    }
                }
            }

            BirdLifecycle::FlyingToNest => {
                if let Some(nav_node) = nesting.nest_nav_node {
                    let nest_pos = nav.node_position(nav_node, &global_transforms);
                    let dist = transform.translation.distance(nest_pos);

                    if dist < Config::NEST_ARRIVAL_DISTANCE {
                        if nesting.nest.is_none() {
                            // First arrival: start building
                            nesting.phase = BirdLifecycle::Building;
                            nesting.timer = Config::NEST_BUILD_TIME;
                        } else {
                            // Returning to existing nest
                            let nest_entity = nesting.nest.unwrap();
                            if let Ok(nest) = nests.get(nest_entity) {
                                if !nest.has_egg && !nest.has_hatchling {
                                    // Lay egg
                                    nesting.phase = BirdLifecycle::Incubating;
                                    nesting.timer = Config::EGG_HATCH_TIME;
                                    commands.entity(nest_entity).insert(Nest {
                                        parent_bird: bird_entity,
                                        nav_node,
                                        has_egg: true,
                                        has_hatchling: false,
                                    });
                                } else {
                                    // Delivering food — back to parenting
                                    nesting.phase = BirdLifecycle::Parenting;
                                }
                            } else {
                                // Nest gone, back to hunting
                                info!("[nesting] Bird {:?} nest gone, back to Hunting", bird_entity);
                                nesting.phase = BirdLifecycle::Hunting;
                                nesting.nest = None;
                            }
                        }
                        // Stop moving
                        velocity.0 = velocity.0.normalize_or_zero() * Config::BIRD_SPEED * 0.1;
                    }
                }
            }

            BirdLifecycle::Building => {
                nesting.timer -= dt;
                if nesting.timer <= 0.0 {
                    // Spawn nest entity, parented to branch so it moves with wind
                    if let Some(nav_node) = nesting.nest_nav_node {
                        let branch_entity = nav.nodes[nav_node].entity;
                        let nest_entity = commands.spawn((
                            Nest {
                                parent_bird: bird_entity,
                                nav_node,
                                has_egg: false,
                                has_hatchling: false,
                            },
                            Transform::default(),
                        )).id();
                        // Parent to branch entity if available, otherwise use world position
                        if let Some(parent) = branch_entity {
                            commands.entity(parent).add_child(nest_entity);
                        } else {
                            let nest_pos = nav.node_position(nav_node, &global_transforms);
                            commands.entity(nest_entity).insert(
                                Transform::from_translation(nest_pos),
                            );
                        }
                        nesting.nest = Some(nest_entity);
                        nesting.phase = BirdLifecycle::HuntingForEgg;
                        velocity.0 = velocity.0.normalize_or_zero() * Config::BIRD_SPEED;
                    }
                }
            }

            BirdLifecycle::HuntingForEgg => {
                if ate_insect {
                    nesting.phase = BirdLifecycle::FlyingToNest;
                }
            }

            BirdLifecycle::Incubating => {
                nesting.timer -= dt;
                if nesting.timer <= 0.0 {
                    // Hatch: spawn hatchling as child of nest (inherits branch transform)
                    if let Some(nest_entity) = nesting.nest {
                        if let Some(nav_node) = nesting.nest_nav_node {
                            let hatchling_entity = commands.spawn((
                                Hatchling {
                                    nest: nest_entity,
                                    parent_bird: bird_entity,
                                    alert: false,
                                },
                                Transform::from_translation(Vec3::new(0.0, 1.5, 0.0)),
                            )).id();
                            commands.entity(nest_entity).add_child(hatchling_entity);
                            commands.entity(nest_entity).insert(Nest {
                                parent_bird: bird_entity,
                                nav_node,
                                has_egg: false,
                                has_hatchling: true,
                            });
                            nesting.phase = BirdLifecycle::Parenting;
                            velocity.0 = velocity.0.normalize_or_zero() * Config::BIRD_SPEED;
                        }
                    }
                }
            }

            BirdLifecycle::Parenting => {
                if ate_insect {
                    nesting.insects_eaten += 1;
                    if nesting.insects_eaten % 2 == 0 {
                        nesting.phase = BirdLifecycle::FlyingToNest;
                    }
                }
            }

            BirdLifecycle::Defending => {
                if let Some(nav_node) = nesting.nest_nav_node {
                    let nest_pos = nav.node_position(nav_node, &global_transforms);
                    let dist = transform.translation.distance(nest_pos);
                    if dist < Config::NEST_ARRIVAL_DISTANCE {
                        nesting.phase = BirdLifecycle::Parenting;
                        velocity.0 = velocity.0.normalize_or_zero() * Config::BIRD_SPEED;
                    }
                }
            }
        }

        // Log transitions
        if nesting.phase != prev_phase {
            info!(
                "[nesting] Bird {:?}: {:?} -> {:?}",
                bird_entity, prev_phase, nesting.phase
            );
        }
    }
}

/// Steer birds toward their nest when in FlyingToNest or Defending phase.
pub fn bird_fly_to_target_system(
    nav: Res<NavGraph>,
    global_transforms: Query<&GlobalTransform>,
    mut birds: Query<(&Transform, &mut Velocity, &BirdNestingState), With<Bird>>,
) {
    for (transform, mut velocity, nesting) in &mut birds {
        let speed_mult = match nesting.phase {
            BirdLifecycle::FlyingToNest => 1.0,
            BirdLifecycle::Defending => Config::BIRD_DEFEND_SPEED_MULT,
            _ => continue,
        };

        let Some(nav_node) = nesting.nest_nav_node else { continue };
        let nest_pos = nav.node_position(nav_node, &global_transforms);
        let to_nest = nest_pos - transform.translation;
        let dist = to_nest.length();

        if dist > f32::EPSILON {
            let dir = to_nest / dist;
            velocity.0 = dir * Config::BIRD_SPEED * speed_mult;
        }
    }
}

/// Check if squirrels are near hatchlings; set alert and trigger parent defense.
pub fn hatchling_alert_system(
    mut hatchlings: Query<(&GlobalTransform, &mut Hatchling)>,
    mut birds: Query<(&mut BirdNestingState, &mut Velocity), With<Bird>>,
    squirrels: Query<(&GlobalTransform, &SquirrelTarget), With<Squirrel>>,
) {
    for (hatchling_transform, mut hatchling) in &mut hatchlings {
        let mut under_threat = false;

        for (sq_transform, _target) in &squirrels {
            let dist = sq_transform.translation().distance(hatchling_transform.translation());
            if dist < Config::HATCHLING_ALERT_RADIUS {
                under_threat = true;
                break;
            }
        }

        if under_threat && !hatchling.alert {
            hatchling.alert = true;
            info!("[nesting] Hatchling alerting! parent={:?}", hatchling.parent_bird);
            if let Ok((mut nesting, mut velocity)) = birds.get_mut(hatchling.parent_bird) {
                if nesting.phase != BirdLifecycle::Defending {
                    info!("[nesting] Bird {:?}: {:?} -> Defending", hatchling.parent_bird, nesting.phase);
                    nesting.phase = BirdLifecycle::Defending;
                    velocity.0 = velocity.0.normalize_or_zero() * Config::BIRD_SPEED;
                }
            }
        } else if !under_threat {
            hatchling.alert = false;
        }
    }
}
