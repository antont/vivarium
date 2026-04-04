use bevy::ecs::message::MessageReader;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

/// Marker + state for the orbit camera.
#[derive(Component)]
pub struct OrbitCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub yaw: f32,   // radians
    pub pitch: f32,  // radians
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            radius: 500.0,
            yaw: 0.0,
            pitch: std::f32::consts::FRAC_PI_4, // 45 degrees
        }
    }
}

const MOUSE_SENSITIVITY: f32 = 0.005;
const KEYBOARD_ROTATE_SPEED: f32 = 2.0;
const ZOOM_SPEED: f32 = 20.0;
const MIN_RADIUS: f32 = 20.0;
const MAX_RADIUS: f32 = 1500.0;
const MIN_PITCH: f32 = 0.05;
const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.05;

pub fn orbit_camera_system(
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut scroll: MessageReader<MouseWheel>,
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (mut orbit, mut transform) in &mut query {
        // Mouse drag: left or right button to orbit
        if mouse_button.pressed(MouseButton::Left) || mouse_button.pressed(MouseButton::Right) {
            for ev in mouse_motion.read() {
                orbit.yaw -= ev.delta.x * MOUSE_SENSITIVITY;
                orbit.pitch += ev.delta.y * MOUSE_SENSITIVITY;
            }
        } else {
            mouse_motion.clear();
        }

        // Keyboard rotation
        if keyboard.pressed(KeyCode::ArrowLeft) {
            orbit.yaw += KEYBOARD_ROTATE_SPEED * dt;
        }
        if keyboard.pressed(KeyCode::ArrowRight) {
            orbit.yaw -= KEYBOARD_ROTATE_SPEED * dt;
        }
        if keyboard.pressed(KeyCode::ArrowUp) {
            orbit.pitch += KEYBOARD_ROTATE_SPEED * dt;
        }
        if keyboard.pressed(KeyCode::ArrowDown) {
            orbit.pitch -= KEYBOARD_ROTATE_SPEED * dt;
        }

        // Scroll to zoom
        for ev in scroll.read() {
            let amount = match ev.unit {
                MouseScrollUnit::Line => ev.y * ZOOM_SPEED,
                MouseScrollUnit::Pixel => ev.y * ZOOM_SPEED * 0.1,
            };
            orbit.radius -= amount;
        }

        // Clamp
        orbit.pitch = orbit.pitch.clamp(MIN_PITCH, MAX_PITCH);
        orbit.radius = orbit.radius.clamp(MIN_RADIUS, MAX_RADIUS);

        // Compute camera position from spherical coordinates
        let x = orbit.radius * orbit.pitch.cos() * orbit.yaw.sin();
        let y = orbit.radius * orbit.pitch.sin();
        let z = orbit.radius * orbit.pitch.cos() * orbit.yaw.cos();

        let eye = orbit.focus + Vec3::new(x, y, z);
        *transform = Transform::from_translation(eye).looking_at(orbit.focus, Vec3::Y);
    }
}
