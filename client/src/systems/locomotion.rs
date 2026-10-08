#![allow(dead_code)]

use bevy::prelude::*;
use crate::components::LocalCharacter;
use crate::plugins::terrain_plugin::terrain_height;

/// Procedural locomotion in the Overgrowth spirit: the body rides a smoothed
/// slope-aligned up vector (sample fore/aft/left/right like the reference
/// `slope align` pass), bobs with speed, and leans into motion. glTF clip
/// playback (AnimationPlayer) layers on top via `animation.rs`.
///
/// Rotation ownership: `player_input` owns `yaw` for the local character;
/// remote bodies derive it from velocity here. This system alone composes
/// the final orientation — nobody else touches rotation.
///
/// Per-foot IK is NOT here: it needs joint access, which arrives with the
/// asset pipeline and the spec skeleton (see CHARACTER_SPEC.md). Slope
/// alignment covers the feel until then.
#[derive(Component, Debug, Clone)]
pub struct Locomotion {
    pub base_y: f32,
    pub phase: f32,
    pub last_pos: Vec3,
    /// Bob amplitude in meters.
    pub bob_amp: f32,
    /// Smoothed velocity (controller-owned for local, derived for remote).
    pub vel: Vec3,
    /// Facing yaw. Controller-owned for local, velocity-derived for remote.
    pub yaw: f32,
    /// Smoothed terrain up vector.
    pub slope_up: Vec3,
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
            slope_up: Vec3::Y,
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

/// Compose the body orientation from facing yaw + slope up + speed lean.
/// The basis is re-orthonormalized and the result normalized: Bevy's axis
/// helpers (`forward()` etc.) debug-panic on denormalized rotations, and a
/// tilted cross product is exactly how you get one. Pure for testing.
pub fn compose_body(yaw: f32, slope_up: Vec3, speed: f32) -> Quat {
    let fwd = Vec3::new(yaw.sin(), 0.0, yaw.cos());
    let mut right = slope_up.cross(fwd);
    if right.length_squared() < 1e-8 {
        right = Vec3::X;
    } else {
        right = right.normalize();
    }
    let up = slope_up.normalize_or_zero();
    let up = if up == Vec3::ZERO { Vec3::Y } else { up };
    let fwd2 = right.cross(up).normalize_or_zero();
    let fwd2 = if fwd2 == Vec3::ZERO { fwd } else { fwd2 };
    // Re-orthogonalize: right stays, up rebuilt perpendicular.
    let up2 = fwd2.cross(right).normalize_or_zero();
    let up2 = if up2 == Vec3::ZERO { Vec3::Y } else { up2 };
    let right2 = up2.cross(fwd2).normalize_or_zero();
    let right2 = if right2 == Vec3::ZERO { Vec3::X } else { right2 };
    let lean = (speed * 0.02).clamp(0.0, 0.15);
    let base = Mat3::from_cols(right2, up2, -fwd2);
    (Quat::from_axis_angle(right2, lean) * Quat::from_mat3(&base)).normalize()
}

/// Per-frame body update: smooth the slope up, compose orientation, ride
/// terrain height with a speed bob.
fn pose(transform: &mut Transform, loco: &mut Locomotion, speed: f32, dt: f32) {
    let p = transform.translation;
    // Slope samples around the feet, same cross pattern as the reference.
    let fx = loco.yaw.sin();
    let fz = loco.yaw.cos();
    let rx = fz;
    let rz = -fx;
    let ha = terrain_height(p.x + fx * 0.8, p.z + fz * 0.8);
    let hb = terrain_height(p.x - fx * 0.8, p.z - fz * 0.8);
    let hl = terrain_height(p.x + rx * 0.8, p.z + rz * 0.8);
    let hr = terrain_height(p.x - rx * 0.8, p.z - rz * 0.8);
    let dhf = ((ha - hb) / 1.6).clamp(-0.6, 0.6);
    let dhs = ((hl - hr) / 1.6).clamp(-0.6, 0.6);
    let desired_up = Vec3::new(-dhs, 1.0, -dhf).normalize();
    let k = 1.0 - (-8.0 * dt).exp();
    loco.slope_up = (loco.slope_up * (1.0 - k) + desired_up * k).normalize();

    // Basis: yaw facing, tilted by the smoothed slope up.
    transform.rotation = compose_body(loco.yaw, loco.slope_up, speed);

    // Ground follows the feet: re-sample every frame so hills, dips, and
    // root-motion travel never leave the body floating or buried.
    loco.base_y = terrain_height(p.x, p.z);
    if speed > 0.5 {
        loco.phase += dt * (4.0 + speed * 1.2);
        let bob = loco.phase.sin().abs() * loco.bob_amp * (speed.min(14.0) / 6.0).min(1.4);
        transform.translation.y = loco.base_y + bob;
    } else {
        // Ground follows the feet: re-sample every frame so hills, dips,
        // and root-motion travel never leave the body floating or buried.
        loco.base_y = terrain_height(p.x, p.z);
        transform.translation.y += (loco.base_y - transform.translation.y) * (dt * 8.0).min(1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_compose_never_denormalizes() {
        // Yaw sweep × slope tilts including degenerate straight-up input.
        let ups = [
            Vec3::Y,
            Vec3::new(0.3, 0.9, -0.2),
            Vec3::new(-0.5, 0.7, 0.5),
            Vec3::new(0.0, 0.2, 0.0),
            Vec3::ZERO,
            Vec3::new(0.0, -1.0, 0.0),
        ];
        for i in 0..64 {
            let yaw = i as f32 * 0.314159;
            for up in ups {
                for speed in [0.0, 3.0, 13.8] {
                    let q = compose_body(yaw, up, speed);
                    assert!(
                        q.is_normalized(),
                        "denormalized at yaw={yaw} up={up:?} speed={speed}"
                    );
                    // Forward stays sane (no NaN, roughly horizontal).
                    let f = q * Vec3::NEG_Z;
                    assert!(f.is_finite());
                    assert!(f.y.abs() < 0.9, "forward tipped over: {f:?}");
                }
            }
        }
    }
}
