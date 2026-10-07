#![allow(dead_code)]

use bevy::prelude::*;
use crate::components::LocalCharacter;

/// Procedural locomotion: bob + lean while moving, settle when idle.
/// glTF clip playback (AnimationPlayer) arrives with the asset pipeline —
/// this keeps everything alive until then, with zero asset plumbing.
///
/// Rotation ownership: remote entities face their velocity here; the LOCAL
/// character's yaw belongs to the controller (`player_input`), so this
/// system only bobs it.
#[derive(Component, Debug, Clone)]
pub struct Locomotion {
    pub base_y: f32,
    pub phase: f32,
    pub last_pos: Vec3,
    /// Bob amplitude in meters; lean factor in radians per (m/s).
    pub bob_amp: f32,
    /// Smoothed velocity (controller-owned for local, derived for remote).
    pub vel: Vec3,
    // --- dodge state (local player only) ---
    pub dodge_t: f32,
    pub dodge_cd: f32,
    pub dodge_dir: Vec3,
    pub tap_key: u8,
    pub tap_t: f32,
}

impl Locomotion {
    pub fn new(base_y: f32, pos: Vec3, bob_amp: f32) -> Self {
        Self {
            base_y,
            phase: 0.0,
            last_pos: pos,
            bob_amp,
            vel: Vec3::ZERO,
            dodge_t: 0.0,
            dodge_cd: 0.0,
            dodge_dir: Vec3::ZERO,
            tap_key: 0,
            tap_t: 0.0,
        }
    }
}

pub fn animate_locomotion(
    time: Res<Time>,
    mut remote: Query<(&mut Transform, &mut Locomotion), Without<LocalCharacter>>,
    mut local: Query<(&mut Transform, &mut Locomotion), With<LocalCharacter>>,
) {
    let dt = time.delta_secs().max(0.0001);
    // Remote bodies: face travel direction, bob, lean.
    for (mut transform, mut loco) in &mut remote {
        let planar = Vec3::new(
            transform.translation.x - loco.last_pos.x,
            0.0,
            transform.translation.z - loco.last_pos.z,
        );
        let speed = planar.length() / dt;
        loco.last_pos = transform.translation;
        loco.vel = planar / dt;

        if speed > 0.5 {
            loco.phase += dt * (4.0 + speed * 1.2);
            let bob = loco.phase.sin().abs() * loco.bob_amp * (speed.min(6.0) / 6.0);
            transform.translation.y = loco.base_y + bob;
            let dir = planar.normalize_or_zero();
            if dir != Vec3::ZERO {
                let yaw = dir.x.atan2(dir.z);
                let pitch = (speed * 0.03).clamp(0.0, 0.18);
                transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
            }
        } else {
            settle(&mut transform, &mut loco, dt);
        }
    }
    // Local body: yaw belongs to the controller; bob + settle only.
    for (mut transform, mut loco) in &mut local {
        let planar = Vec3::new(
            transform.translation.x - loco.last_pos.x,
            0.0,
            transform.translation.z - loco.last_pos.z,
        );
        let speed = planar.length() / dt;
        loco.last_pos = transform.translation;
        if speed > 0.5 {
            loco.phase += dt * (4.0 + speed * 1.2);
            let bob = loco.phase.sin().abs() * loco.bob_amp * (speed.min(14.0) / 6.0).min(1.4);
            transform.translation.y = loco.base_y + bob;
        } else {
            settle(&mut transform, &mut loco, dt);
        }
    }
}

fn settle(transform: &mut Transform, loco: &mut Locomotion, dt: f32) {
    transform.translation.y += (loco.base_y - transform.translation.y) * (dt * 8.0).min(1.0);
    let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
    let eased = pitch * (1.0 - (dt * 8.0).min(1.0));
    transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, eased, 0.0);
}
