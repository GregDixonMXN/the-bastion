#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use std::time::Instant;
use crate::components::LocalCharacter;
use super::locomotion::Locomotion;

/// Dead-simple third-person camera. Every frame, from scratch:
/// position = character + orbit offset, look at character.
/// No accumulated anchor state, so there is nothing that can get stuck.
/// Right-mouse drag orbits; otherwise it trails behind your heading.
#[derive(Resource, Debug, Clone)]
pub struct OrbitCam {
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for OrbitCam {
    fn default() -> Self {
        Self { yaw: 0.0, pitch: -0.25 }
    }
}

/// Camera forward on the ground plane, shared with the controller so W
/// always means "away from camera".
#[derive(Resource, Debug, Clone, Default)]
pub struct FacingCam {
    pub yaw: f32,
}

const CAM_DIST: f32 = 8.0;
const CAM_HEIGHT: f32 = 2.2;
const MOUSE_SENS: f32 = 0.004;

pub fn orbit_camera(
    mut mouse: EventReader<MouseMotion>,
    buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut orbit: ResMut<OrbitCam>,
    mut facing: ResMut<FacingCam>,
    char_query: Query<(&Transform, &Locomotion), With<LocalCharacter>>,
    mut cam_query: Query<&mut Transform, (With<Camera3d>, Without<LocalCharacter>)>,
) {
    if buttons.pressed(MouseButton::Right) {
        for event in mouse.read() {
            orbit.yaw -= event.delta.x * MOUSE_SENS;
            orbit.pitch -= event.delta.y * MOUSE_SENS * 0.8;
            orbit.pitch = orbit.pitch.clamp(-1.2, 0.5);
        }
    } else {
        // Trail: drift behind the heading while running hands-free.
        if let Ok((ct, loco)) = char_query.single() {
            if loco.vel.length() > 1.0 {
                let f = ct.rotation.normalize().mul_vec3(Vec3::NEG_Z);
                let char_yaw = (-f.x).atan2(-f.z);
                let want = char_yaw + std::f32::consts::PI;
                let mut d = want - orbit.yaw;
                while d < -std::f32::consts::PI {
                    d += 2.0 * std::f32::consts::PI;
                }
                while d > std::f32::consts::PI {
                    d -= 2.0 * std::f32::consts::PI;
                }
                orbit.yaw += d * (1.0 - (-3.0 * time.delta_secs()).exp());
            }
        }
        // Drain unread motion so it never leaks into the next drag.
        for _ in mouse.read() {}
    }
    facing.yaw = orbit.yaw;

    let focus = if let Ok((transform, _)) = char_query.single() {
        transform.translation + Vec3::new(0.0, 1.6, 0.0)
    } else {
        Vec3::new(0.0, 2.0, -18.0)
    };
    let (cp, sp) = (orbit.pitch.cos(), orbit.pitch.sin());
    let offset = Vec3::new(orbit.yaw.sin() * cp, -sp + CAM_HEIGHT / CAM_DIST, orbit.yaw.cos() * cp)
        * CAM_DIST;
    let want = focus + offset;

    if let Ok(mut cam_tf) = cam_query.single_mut() {
        // Critically damped follow: fast, no state to wedge.
        let k = 1.0 - (-12.0 * time.delta_secs()).exp();
        let pos = cam_tf.translation;
        cam_tf.translation = pos + (want - pos) * k;
        cam_tf.look_at(focus, Vec3::Y);
    }
}
