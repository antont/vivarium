use crate::components::{
    Bird, BirdLifecycle, BirdNestingState, HuntPhase, HuntState, Insect, Predator, Velocity, Wander,
};
use crate::config::Config;
use crate::spatial::SpatialIndex;
use crate::wind::Wind;
use bevy::prelude::*;
use rand::Rng;

pub fn hunt_system(
    time: Res<Time>,
    wind: Res<Wind>,
    spatial: Res<SpatialIndex>,
    mut birds: Query<
        (
            &Transform,
            &mut Velocity,
            &Predator,
            &mut HuntState,
            &Wander,
            &BirdNestingState,
        ),
        With<Bird>,
    >,
    insects: Query<(Entity, &Transform), With<Insect>>,
) {
    let dt = time.delta_secs();
    let mut rng = rand::rng();

    for (bird_transform, mut velocity, predator, mut hunt, wander, nesting) in &mut birds {
        // Skip birds not in a hunting-capable lifecycle phase.
        // (Can't use QueryFilter here — phase is an enum field, not a component.)
        match nesting.phase {
            BirdLifecycle::Hunting
            | BirdLifecycle::HuntingForEgg
            | BirdLifecycle::Parenting => {}
            _ => continue,
        }
        let bird_pos = bird_transform.translation;
        let forward = velocity.0.normalize_or_zero();

        // Find closest visible insect
        let mut closest_insect: Option<(Vec3, f32)> = None;
        if forward != Vec3::ZERO {
            let nearby = spatial.get_nearby(bird_pos);
            for &nearby_entity in &nearby {
                let Ok((_, insect_transform)) = insects.get(nearby_entity) else {
                    continue;
                };
                let to_insect = insect_transform.translation - bird_pos;
                let dist = to_insect.length();
                if dist > predator.sight_range || dist < f32::EPSILON {
                    continue;
                }
                let to_norm = to_insect / dist;
                let angle = forward.dot(to_norm).clamp(-1.0, 1.0).acos();
                if angle < predator.sight_half_angle {
                    if closest_insect.is_none() || dist < closest_insect.unwrap().1 {
                        closest_insect = Some((insect_transform.translation, dist));
                    }
                }
            }
        }

        match hunt.phase {
            HuntPhase::Searching => {
                if let Some((target_pos, _)) = closest_insect {
                    // Spotted prey — start circling
                    hunt.phase = HuntPhase::Circling;
                    hunt.timer = 0.0;
                    hunt.target_pos = target_pos;
                } else {
                    // Wander: gentle random rotation
                    let random_axis = Vec3::new(
                        rng.random_range(-1.0..1.0),
                        rng.random_range(-1.0..1.0),
                        rng.random_range(-1.0..1.0),
                    )
                    .normalize_or_zero();
                    if random_axis != Vec3::ZERO {
                        let angle = rng.random_range(-wander.strength..wander.strength);
                        let rotation = Quat::from_axis_angle(random_axis, angle.to_radians());
                        let speed = velocity.0.length();
                        velocity.0 = (rotation * velocity.0).normalize_or_zero() * speed;
                    }
                }
            }

            HuntPhase::Circling => {
                hunt.timer += dt;

                // Update target if we can still see insects
                if let Some((target_pos, _)) = closest_insect {
                    hunt.target_pos = target_pos;
                }

                // Circle around target, compensating for wind drift differential
                let drift = wind.relative_drift();
                let to_target = (hunt.target_pos + drift * 0.5) - bird_pos;
                let dist_to_target = to_target.length();

                if dist_to_target > f32::EPSILON {
                    let to_target_norm = to_target / dist_to_target;
                    // Perpendicular direction (cross with up, fallback to another axis)
                    let perp = to_target_norm.cross(Vec3::Y).normalize_or_zero();
                    let perp = if perp == Vec3::ZERO {
                        to_target_norm.cross(Vec3::X).normalize_or_zero()
                    } else {
                        perp
                    };

                    // Blend: mostly tangential + some radial pull toward circle radius
                    let radial = if dist_to_target > Config::HUNT_CIRCLE_RADIUS {
                        to_target_norm * 0.5
                    } else {
                        -to_target_norm * 0.3
                    };

                    let steer = perp * 2.0 + radial;
                    let speed = velocity.0.length();
                    velocity.0 = (velocity.0 + steer).normalize_or_zero() * speed;
                }

                // Transition to diving after circle duration
                if hunt.timer >= Config::HUNT_CIRCLE_DURATION {
                    hunt.phase = HuntPhase::Diving;
                }
            }

            HuntPhase::Diving => {
                // Dive toward target, leading by wind drift differential
                let to_target = hunt.target_pos - bird_pos;
                let dist = to_target.length();

                if dist > f32::EPSILON {
                    let dive_speed = Config::BIRD_SPEED * Config::HUNT_DIVE_SPEED_MULT;
                    let time_to_reach = dist / dive_speed;
                    let drift = wind.relative_drift();
                    let lead_pos = hunt.target_pos + drift * time_to_reach;
                    let dive_dir = (lead_pos - bird_pos).normalize_or_zero();
                    velocity.0 = dive_dir * dive_speed;
                }

                // Reset to searching after getting close or target eaten
                if dist < Config::BIRD_EATING_DISTANCE * 2.0 || closest_insect.is_none() {
                    hunt.phase = HuntPhase::Searching;
                    hunt.timer = 0.0;
                    // Restore normal speed
                    velocity.0 = velocity.0.normalize_or_zero() * Config::BIRD_SPEED;
                }
            }
        }
    }
}
