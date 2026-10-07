#![allow(dead_code)]

use bevy::prelude::*;

/// Procedural locomotion: bob + lean while moving, settle when idle.
/// glTF clip playback (AnimationPlayer) arrives with the asset pipeline —
/// this keeps everything alive until then, with zero asset plumbing.
#[derive(Component, Debug, Clone)]
pub struct Locomotion {
    pub base_y: f32,
    pub phase: f32,
    pub last_pos: Vec3,
    /// Bob amplitude in meters; lean factor in radians per (m/s).
    pub bob_amp: f32,
}

impl Locomotion {
    pub fn new(base_y: f32, pos: Vec3, bob_amp: f32) -> Self {
        Self { base_y, phase: 0.0, last_pos: pos, bob_amp }
    }
}

pub fn animate_locomotion(time: Res<Time>, mut query: Query<(&mut Transform, &mut Locomotion)>) {
    let dt = time.delta_secs().max(0.0001);
    for (mut transform, mut loco) in &mut query {
        let planar = Vec3::new(
            transform.translation.x - loco.last_pos.x,
            0.0,
            transform.translation.z - loco.last_pos.z,
        );
        let speed = planar.length() / dt;
        loco.last_pos = transform.translation;

        if speed > 0.5 {
            loco.phase += dt * (4.0 + speed * 1.2);
            let bob = loco.phase.sin().abs() * loco.bob_amp * speed.min(6.0) / 6.0;
            transform.translation.y = loco.base_y + bob;
            // Lean into the run.
            let dir = planar.normalize_or_zero();
            if dir != Vec3::ZERO {
                let yaw = dir.x.atan2(dir.z);
                let pitch = (speed * 0.03).clamp(0.0, 0.18);
                transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
            }
        } else {
            // Settle back to rest pose.
            transform.translation.y += (loco.base_y - transform.translation.y) * (dt * 8.0).min(1.0);
            let (_, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            let eased = pitch * (1.0 - (dt * 8.0).min(1.0));
            let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
            transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, eased, 0.0);
        }
    }
}
