use bevy::ecs::message::MessageReader;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use crate::components::{Squirrel, SquirrelIndex};

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

/// Which camera mode is active.
#[derive(Resource)]
pub enum CameraMode {
    /// Free orbit camera with WASD strafing
    Overview,
    /// Follow a specific squirrel by index
    FollowSquirrel(usize),
}

impl Default for CameraMode {
    fn default() -> Self {
        Self::Overview
    }
}

const MOUSE_SENSITIVITY: f32 = 0.005;
const KEYBOARD_ROTATE_SPEED: f32 = 2.0;
const ZOOM_SPEED: f32 = 20.0;
const STRAFE_SPEED: f32 = 200.0;
const MIN_RADIUS: f32 = 20.0;
const MAX_RADIUS: f32 = 1500.0;
const MIN_PITCH: f32 = 0.05;
const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.05;

const FOLLOW_RADIUS: f32 = 40.0;
const FOLLOW_PITCH: f32 = 0.5; // ~29 degrees, fairly low angle
const FOLLOW_SMOOTH: f32 = 4.0;

/// Switch camera mode based on key presses.
pub fn camera_mode_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<CameraMode>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        *mode = CameraMode::Overview;
    }
    if keyboard.just_pressed(KeyCode::KeyZ) {
        *mode = CameraMode::FollowSquirrel(0);
    }
    if keyboard.just_pressed(KeyCode::KeyX) {
        *mode = CameraMode::FollowSquirrel(1);
    }
    if keyboard.just_pressed(KeyCode::KeyC) {
        *mode = CameraMode::FollowSquirrel(2);
    }
}

pub fn orbit_camera_system(
    time: Res<Time>,
    mode: Res<CameraMode>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut scroll: MessageReader<MouseWheel>,
    mut camera_q: Query<(&mut OrbitCamera, &mut Transform)>,
    squirrels: Query<(&Transform, &SquirrelIndex), (With<Squirrel>, Without<OrbitCamera>)>,
) {
    let dt = time.delta_secs();

    for (mut orbit, mut transform) in &mut camera_q {
        match *mode {
            CameraMode::Overview => {
                // Mouse drag: left or right button to orbit
                if mouse_button.pressed(MouseButton::Left) || mouse_button.pressed(MouseButton::Right) {
                    for ev in mouse_motion.read() {
                        orbit.yaw -= ev.delta.x * MOUSE_SENSITIVITY;
                        orbit.pitch += ev.delta.y * MOUSE_SENSITIVITY;
                    }
                } else {
                    mouse_motion.clear();
                }

                // Keyboard rotation (arrows)
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

                // WASD strafing — move focus point relative to camera facing
                let mut strafe = Vec3::ZERO;
                if keyboard.pressed(KeyCode::KeyW) {
                    strafe.z -= 1.0;
                }
                if keyboard.pressed(KeyCode::KeyS) {
                    strafe.z += 1.0;
                }
                if keyboard.pressed(KeyCode::KeyA) {
                    strafe.x -= 1.0;
                }
                if keyboard.pressed(KeyCode::KeyD) {
                    strafe.x += 1.0;
                }
                if strafe != Vec3::ZERO {
                    strafe = strafe.normalize();
                    // Rotate strafe direction by camera yaw (horizontal plane)
                    let forward = Vec3::new(-orbit.yaw.sin(), 0.0, -orbit.yaw.cos());
                    let right = Vec3::new(forward.z, 0.0, -forward.x);
                    orbit.focus += (forward * -strafe.z + right * strafe.x) * STRAFE_SPEED * dt;
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
            }

            CameraMode::FollowSquirrel(idx) => {
                mouse_motion.clear();
                scroll.clear();

                // Find the squirrel with matching index
                let target = squirrels.iter()
                    .find(|(_, si)| si.0 == idx)
                    .map(|(t, _)| t.translation);

                if let Some(target_pos) = target {
                    // Smoothly move focus to squirrel
                    orbit.focus = orbit.focus.lerp(target_pos, FOLLOW_SMOOTH * dt);
                    orbit.radius = orbit.radius + (FOLLOW_RADIUS - orbit.radius) * (FOLLOW_SMOOTH * dt).min(1.0);
                    orbit.pitch = orbit.pitch + (FOLLOW_PITCH - orbit.pitch) * (FOLLOW_SMOOTH * dt).min(1.0);
                }

                // Still allow mouse orbit in follow mode
                if mouse_button.pressed(MouseButton::Left) || mouse_button.pressed(MouseButton::Right) {
                    for ev in mouse_motion.read() {
                        orbit.yaw -= ev.delta.x * MOUSE_SENSITIVITY;
                        orbit.pitch += ev.delta.y * MOUSE_SENSITIVITY;
                    }
                }
                orbit.pitch = orbit.pitch.clamp(MIN_PITCH, MAX_PITCH);
                orbit.radius = orbit.radius.clamp(MIN_RADIUS, MAX_RADIUS);
            }
        }

        // Compute camera position from spherical coordinates
        let x = orbit.radius * orbit.pitch.cos() * orbit.yaw.sin();
        let y = orbit.radius * orbit.pitch.sin();
        let z = orbit.radius * orbit.pitch.cos() * orbit.yaw.cos();

        let eye = orbit.focus + Vec3::new(x, y, z);
        *transform = Transform::from_translation(eye).looking_at(orbit.focus, Vec3::Y);
    }
}
