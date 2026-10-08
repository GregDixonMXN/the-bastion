#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use std::time::Duration;
use crate::components::{Character, LocalCharacter};
use crate::generated::move_character_reducer::move_character;
use crate::resources::NetState;
use crate::systems::camera::FacingCam;
use super::locomotion::Locomotion;

/// Root-motion controller: input steers facing and selects gait, the
/// animation's root displacement moves the body. Intent velocity mirrors
/// world speed for clip selection, camera trail, and sync throttle.
/// Shift sprints (run gait). Positions sync under the 15 m/step rule.
pub const WALK_SPEED: f32 = 1.61;
pub const RUN_SPEED: f32 = 4.04;
pub const DODGE_TIME: f32 = 0.22;
pub const DODGE_CD: f32 = 0.7;
const SYNC_DISTANCE: f32 = 0.5;
const SYNC_INTERVAL: Duration = Duration::from_millis(200);

pub fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    facing: Res<FacingCam>,
    mut net: ResMut<NetState>,
    mut query: Query<(&mut Transform, &Character, &mut Locomotion), With<LocalCharacter>>,
) {
    let Ok((transform, character, mut loco)) = query.single_mut() else { return };

    if character.is_dead {
        return; // dead — press R to respawn at camp
    }
    let dt = time.delta_secs().max(0.0001);

    // Camera-relative wish direction (W = away from camera).
    let fwd = Vec3::new(-facing.yaw.sin(), 0.0, -facing.yaw.cos());
    let right = Vec3::new(-fwd.z, 0.0, fwd.x);
    let mut wish = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyW) { wish += fwd; }
    if keyboard.pressed(KeyCode::KeyS) { wish -= fwd; }
    if keyboard.pressed(KeyCode::KeyD) { wish += right; }
    if keyboard.pressed(KeyCode::KeyA) { wish -= right; }
    let wish = if wish == Vec3::ZERO { wish } else { wish.normalize() };

    // Double-tap dodge: same key twice within 0.28 s = dash + brief freedom.
    if loco.tap_t > 0.0 {
        loco.tap_t -= dt;
    }
    if loco.dodge_cd > 0.0 {
        loco.dodge_cd -= dt;
    }
    for (key, dir) in [
        (1u8, fwd),
        (2u8, -fwd),
        (3u8, right),
        (4u8, -right),
    ] {
        let code = match key {
            1 => KeyCode::KeyW,
            2 => KeyCode::KeyS,
            3 => KeyCode::KeyD,
            _ => KeyCode::KeyA,
        };
        if keyboard.just_pressed(code) {
            if key == loco.tap_key && loco.tap_t > 0.0 && loco.dodge_cd <= 0.0 {
                loco.dodge_t = DODGE_TIME;
                loco.dodge_cd = DODGE_CD;
                loco.dodge_dir = dir;
                loco.tap_key = 0;
                loco.tap_t = 0.0;
            } else {
                loco.tap_key = key;
                loco.tap_t = 0.28;
            }
        }
    }

    if loco.dodge_t > 0.0 {
        // Dash: intent burst along the tapped direction, steering locked.
        // Root motion triples the run displacement to match.
        loco.dodge_t -= dt;
        loco.vel = loco.dodge_dir * RUN_SPEED * 3.0;
        loco.yaw = loco.dodge_dir.x.atan2(loco.dodge_dir.z);
    } else {
        let sprinting =
            keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
        let gait = if sprinting { RUN_SPEED } else { WALK_SPEED };
        loco.vel = wish * gait;
        // Facing follows the wish directly (no inertia needed — root
        // displacement already trails the turn through the clip).
        if wish != Vec3::ZERO {
            let want_yaw = wish.x.atan2(wish.z);
            let mut d = want_yaw - loco.yaw;
            while d < -std::f32::consts::PI {
                d += 2.0 * std::f32::consts::PI;
            }
            while d > std::f32::consts::PI {
                d -= 2.0 * std::f32::consts::PI;
            }
            loco.yaw += d * (1.0 - (-12.0 * dt).exp());
        }
    }

    // Throttled authoritative sync.
    let Some(conn) = net.conn.as_ref() else { return };
    let Some(char_id) = net.own_character_id else { return };
    let dx = transform.translation.x - net.last_sent_pos.0;
    let dz = transform.translation.z - net.last_sent_pos.1;
    if (dx * dx + dz * dz).sqrt() >= SYNC_DISTANCE || net.last_sent_at.elapsed() >= SYNC_INTERVAL {
        match conn.reducers.move_character(char_id, transform.translation.x, transform.translation.z) {
            Ok(()) => {
                net.last_sent_pos = (transform.translation.x, transform.translation.z);
                net.last_sent_at = std::time::Instant::now();
            }
            Err(e) => log::warn!("move send failed: {e}"),
        }
    }
}
