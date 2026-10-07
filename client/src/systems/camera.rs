#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use std::time::Instant;
use crate::components::LocalCharacter;
use super::locomotion::Locomotion;
use crate::plugins::terrain_plugin::terrain_height;

/// Overgrowth-style chase camera: right-mouse steers target yaw/pitch,
/// inertia smooths the lens, the trail assist drifts behind your heading
/// while running hands-free, and terrain walks the dolly in before it buries.
#[derive(Resource, Debug, Clone)]
pub struct OrbitCam {
    pub yaw: f32,
    pub pitch: f32,
    pub target_yaw: f32,
    pub target_pitch: f32,
    pub anchor: Vec3,
    pub anchored: bool,
    pub last_look: Instant,
}

impl Default for OrbitCam {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: -0.20,
            target_yaw: 0.0,
            target_pitch: -0.20,
            anchor: Vec3::ZERO,
            anchored: false,
            last_look: Instant::now(),
        }
    }
}

/// Camera forward on the ground plane, shared with the controller so W
/// always means "away from camera".
#[derive(Resource, Debug, Clone, Default)]
pub struct FacingCam {
    pub yaw: f32,
}

const CAM_DIST: f32 = 7.0;
const MOUSE_SENS: f32 = 1.0;

fn ang_diff(a: f32, b: f32) -> f32 {
    let mut d = b - a;
    while d < -std::f32::consts::PI {
        d += 2.0 * std::f32::consts::PI;
    }
    while d > std::f32::consts::PI {
        d -= 2.0 * std::f32::consts::PI;
    }
    d
}

pub fn orbit_camera(
    mut mouse: EventReader<MouseMotion>,
    buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut orbit: ResMut<OrbitCam>,
    mut facing: ResMut<FacingCam>,
    char_query: Query<(&Transform, &Locomotion), With<LocalCharacter>>,
    mut cam_query: Query<&mut Transform, (With<Camera3d>, Without<LocalCharacter>)>,
) {
    // Right mouse = look (WoW-style). Otherwise the trail assist owns yaw.
    let looking = buttons.pressed(MouseButton::Right);
    for event in mouse.read() {
        if !looking {
            continue;
        }
        orbit.target_yaw -= event.delta.x * 0.0035 * MOUSE_SENS;
        orbit.target_pitch -= event.delta.y * 0.0030 * MOUSE_SENS;
        orbit.target_pitch = orbit.target_pitch.clamp(-1.4, 0.7);
        orbit.last_look = Instant::now();
    }

    let anchor = if let Ok((transform, _)) = char_query.single() {
        let mut a = transform.translation + Vec3::new(0.0, 1.6, 0.0);
        // Trail assist: hands off the mouse while running → drift behind heading.
        if !looking && orbit.last_look.elapsed().as_secs_f32() > 1.0 {
            if let Ok((ct, loco)) = char_query.single() {
                let speed = loco.vel.length();
                if speed > 1.0 {
                    let fwd = ct.forward();
                    let char_yaw = (-fwd.x).atan2(-fwd.z);
                    let want = char_yaw + std::f32::consts::PI;
                    orbit.target_yaw += ang_diff(orbit.target_yaw, want)
                        * (1.0 - (-3.0 * time.delta_secs()).exp());
                }
            }
        }
        // Pitch relaxes toward a slight look-down, same as the reference.
        orbit.target_pitch += (-0.20 - orbit.target_pitch)
            * (1.0 - (-1.5 * time.delta_secs()).exp());
        a
    } else {
        // Nobody bound yet — hold a wide view of camp.
        Vec3::new(0.0, 14.0, -18.0)
    };

    if !orbit.anchored {
        orbit.anchor = anchor;
        orbit.anchored = true;
    }
    let k = 1.0 - (-10.0 * time.delta_secs()).exp();
    let anchor_now = anchor;
    let anchor_old = orbit.anchor;
    orbit.anchor = anchor_old + (anchor_now - anchor_old) * k;

    // Rotation inertia.
    let kk = 1.0 - (-8.0 * time.delta_secs()).exp();
    orbit.yaw += ang_diff(orbit.yaw, orbit.target_yaw) * kk;
    orbit.pitch += (orbit.target_pitch - orbit.pitch) * kk;
    facing.yaw = orbit.yaw;

    // Dolly sits behind the facing direction.
    let (cp, sp) = (orbit.pitch.cos(), orbit.pitch.sin());
    let facing_dir = Vec3::new(-orbit.yaw.sin() * cp, sp, -orbit.yaw.cos() * cp);
    let want = orbit.anchor - facing_dir * CAM_DIST;

    // Terrain collision: walk out from anchor, stop before burying.
    let mut placed = want;
    for s in 1..=20 {
        let t = s as f32 / 20.0;
        let p = orbit.anchor.lerp(want, t);
        if p.y < terrain_height(p.x, p.z) + 0.5 {
            let t_prev = (s - 1) as f32 / 20.0;
            placed = orbit.anchor.lerp(want, t_prev);
            break;
        }
        placed = p;
    }

    if let Ok(mut cam_tf) = cam_query.single_mut() {
        cam_tf.translation = placed;
        cam_tf.look_at(orbit.anchor, Vec3::Y);
    }
}
