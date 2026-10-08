#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use std::time::Duration;
use crate::components::{Character, LocalCharacter};
use crate::generated::move_character_reducer::move_character;
use crate::resources::NetState;
use crate::systems::camera::FacingCam;
use super::locomotion::Locomotion;

/// Overgrowth-style bumper controller: camera-relative acceleration with
/// exponential damping (weight, not teleport Glide), yaw that follows
/// forward motion but holds on backpedal, and double-tap dodge dashes.
/// Positions still sync to the server under the 15 m/step rule.
pub const TOP_SPEED: f32 = 6.0;
const ACCEL: f32 = 60.0;
const DAMPING: f32 = 10.0;
const DODGE_TIME: f32 = 0.22;
const DODGE_CD: f32 = 0.7;
const DODGE_MULT: f32 = 2.3;
const SYNC_DISTANCE: f32 = 0.5;
const SYNC_INTERVAL: Duration = Duration::from_millis(200);

pub fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    facing: Res<FacingCam>,
    mut net: ResMut<NetState>,
    mut query: Query<(&mut Transform, &Character, &mut Locomotion), With<LocalCharacter>>,
) {
    let Ok((mut transform, character, mut loco)) = query.single_mut() else { return };

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
        // Dash: fixed burst, steering locked.
        loco.dodge_t -= dt;
        loco.vel = loco.dodge_dir * TOP_SPEED * DODGE_MULT;
    } else {
        loco.vel += wish * ACCEL * dt;
        loco.vel *= (-DAMPING * dt).exp();
        let speed = loco.vel.length();
        if speed > TOP_SPEED {
            loco.vel *= TOP_SPEED / speed;
        }
    }
    transform.translation += loco.vel * dt;

    // Facing: follow velocity when moving forward-ish; backpedal and strafes
    // hold facing (and with it, the trailing camera). Rotation itself is
    // composed by locomotion — here we only steer `yaw`.
    let speed = loco.vel.length();
    if speed > 0.5 {
        let forwardness = loco.vel.dot(fwd) / speed;
        if forwardness > 0.2 {
            loco.yaw = loco.vel.x.atan2(loco.vel.z);
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
