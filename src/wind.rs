use bevy::prelude::*;
use bevy::color::palettes::css;
use crate::components::{TreeAnchor, TreeSegment};
use crate::config::Config;

/// Marker for the wind indicator UI text.
#[derive(Component)]
pub struct WindIndicator;

#[derive(Resource)]
pub struct Wind {
    pub direction: Vec3,
    pub strength: f32,
    phase_dir: f32,
    phase_str: f32,
}

impl Default for Wind {
    fn default() -> Self {
        Self {
            direction: Vec3::new(1.0, 0.0, 0.3).normalize(),
            strength: Config::WIND_BASE_STRENGTH,
            phase_dir: 0.0,
            phase_str: 0.0,
        }
    }
}

impl Wind {
    /// Current wind vector (direction * strength).
    pub fn vector(&self) -> Vec3 {
        self.direction * self.strength
    }

    /// The rate at which insects drift relative to birds due to wind.
    pub fn relative_drift(&self) -> Vec3 {
        self.vector() * (Config::WIND_INSECT_FACTOR - Config::WIND_BIRD_FACTOR)
    }
}

pub fn wind_update_system(time: Res<Time>, mut wind: ResMut<Wind>) {
    let dt = time.delta_secs();

    // Advance oscillator phases
    wind.phase_dir += Config::WIND_DIR_RATE * dt;
    wind.phase_str += Config::WIND_STR_RATE * dt;

    // Direction: rotate around Y with a secondary vertical wobble
    let yaw = wind.phase_dir.sin() * std::f32::consts::PI; // full sweep over time
    let pitch = (wind.phase_dir * 0.7).cos() * 0.15; // gentle vertical wobble
    wind.direction = Vec3::new(yaw.cos(), pitch, yaw.sin()).normalize();

    // Strength: oscillate around base
    let amplitude = (Config::WIND_MAX_STRENGTH - Config::WIND_BASE_STRENGTH).max(0.0);
    wind.strength = (Config::WIND_BASE_STRENGTH + amplitude * wind.phase_str.sin())
        .clamp(0.0, Config::WIND_MAX_STRENGTH);
}

pub fn wind_tree_system(
    wind: Res<Wind>,
    mut trees: Query<(&mut Transform, &TreeAnchor), With<TreeSegment>>,
) {
    let wind_horizontal = Vec3::new(wind.direction.x, 0.0, wind.direction.z);
    if wind_horizontal.length_squared() < f32::EPSILON {
        return;
    }

    // Bend axis: perpendicular to wind direction in horizontal plane
    let tilt_axis = wind_horizontal.normalize().cross(Vec3::Y);
    if tilt_axis.length_squared() < f32::EPSILON {
        return;
    }
    let tilt_axis = tilt_axis.normalize();
    let bend_per_unit = wind.strength * Config::WIND_TREE_BEND_FACTOR;

    for (mut transform, anchor) in &mut trees {
        // Height above tree root determines how much this segment bends
        let height = anchor.local_offset.y.max(0.0);
        let bend_angle = bend_per_unit * height;
        let bend_rot = Quat::from_axis_angle(tilt_axis, bend_angle);

        // Rotate offset around the tree root — higher parts swing further
        transform.translation = anchor.root + bend_rot * anchor.local_offset;
        // Also tilt the segment itself
        transform.rotation = bend_rot * anchor.base_rotation;
    }
}

pub fn setup_wind_indicator(mut commands: Commands) {
    // Container node in top-right corner
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            right: Val::Px(16.0),
            top: Val::Px(16.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::End,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("Wind: --"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(css::WHITE.into()),
                WindIndicator,
            ));
        });
}

pub fn wind_indicator_system(wind: Res<Wind>, mut query: Query<&mut Text, With<WindIndicator>>) {
    let Ok(mut text) = query.single_mut() else {
        return;
    };

    // Cardinal direction from wind horizontal angle
    let dir = wind.direction;
    let angle = dir.z.atan2(dir.x); // radians, 0 = +X (East)
    let cardinal = match ((angle.to_degrees() + 202.5) / 45.0) as i32 % 8 {
        0 => "W",
        1 => "NW",
        2 => "N",
        3 => "NE",
        4 => "E",
        5 => "SE",
        6 => "S",
        7 => "SW",
        _ => "?",
    };

    // Strength bar: filled squares proportional to strength
    let bars = ((wind.strength / Config::WIND_MAX_STRENGTH) * 5.0).round() as usize;
    let bar_str: String = "|".repeat(bars);

    **text = format!("Wind: {} {:.0} {}", cardinal, wind.strength, bar_str);
}
