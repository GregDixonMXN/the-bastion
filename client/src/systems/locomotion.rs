#![allow(dead_code)]

use bevy::prelude::*;
use crate::components::LocalCharacter;

/// Minimal locomotion state: velocity intent, facing yaw, dodge timers.
/// The ground is flat (y=0); terrain returns with the heightmap pass.
#[derive(Component, Debug, Clone)]
pub struct Locomotion {
    pub base_y: f32,
    pub phase: f32,
    pub last_pos: Vec3,
    /// Bob amplitude in meters (0 for clip-driven bodies).
    pub bob_amp: f32,
    /// Smoothed velocity (controller-owned for local, derived for remote).
    pub vel: Vec3,
    /// Facing yaw. Controller-owned for local, velocity-derived for remote.
    pub yaw: f32,
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
            yaw: 0.0,
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
            let dir = planar.normalize_or_zero();
            if dir != Vec3::ZERO {
                loco.yaw = dir.x.atan2(dir.z);
            }
        }
        pose(&mut transform, &mut loco, speed, dt);
    }
    for (mut transform, mut loco) in &mut local {
        let planar = Vec3::new(
            transform.translation.x - loco.last_pos.x,
            0.0,
            transform.translation.z - loco.last_pos.z,
        );
        let speed = planar.length() / dt;
        loco.last_pos = transform.translation;
        pose(&mut transform, &mut loco, speed, dt);
    }
}

/// Compose orientation from yaw + speed lean, ride flat ground with a bob.
/// Rotation is always re-normalized: Bevy's axis helpers debug-panic on
/// denormalized rotations.
fn pose(transform: &mut Transform, loco: &mut Locomotion, speed: f32, dt: f32) {
    // Model nose is -Z while movement yaw faces +Z-style (sin, cos): offset
    // by half a turn so the body walks nose-first, not butt-first.
    let visual_yaw = loco.yaw + std::f32::consts::PI;
    let lean = (speed * 0.02).clamp(0.0, 0.15);
    transform.rotation = (Quat::from_rotation_y(visual_yaw)
        * Quat::from_axis_angle(Vec3::X, -lean))
    .normalize();

    if speed > 0.5 {
        loco.phase += dt * (4.0 + speed * 1.2);
        let bob = loco.phase.sin().abs() * loco.bob_amp * (speed.min(14.0) / 6.0).min(1.4);
        transform.translation.y = loco.base_y + bob;
    } else {
        transform.translation.y += (loco.base_y - transform.translation.y) * (dt * 8.0).min(1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composed_rotation_stays_normalized() {
        for i in 0..64 {
            let yaw = i as f32 * 0.314159;
            for speed in [0.0f32, 3.0, 13.8] {
                let lean = (speed * 0.02).clamp(0.0, 0.15);
                let q = (Quat::from_rotation_y(yaw)
                    * Quat::from_axis_angle(Vec3::X, lean))
                .normalize();
                assert!(q.is_normalized(), "yaw={yaw} speed={speed}");
            }
        }
    }
}
