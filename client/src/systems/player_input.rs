#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use std::time::Duration;
use crate::components::{Character, LocalCharacter};
use crate::generated::move_character_reducer::move_character;
use crate::resources::NetState;

const MOVE_SPEED: f32 = 6.0;
const SYNC_DISTANCE: f32 = 0.5;
const SYNC_INTERVAL: Duration = Duration::from_millis(200);

/// Reads WASD input, predicts locally, and throttles authoritative position
/// syncs to the server. The server re-checks ownership and range.
pub fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut net: ResMut<NetState>,
    mut query: Query<(&mut Transform, &Character), With<LocalCharacter>>,
) {
    let Ok((mut transform, character)) = query.single_mut() else { return };

    if character.is_dead {
        return; // dead — press R to respawn at camp
    }

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) { direction.z -= 1.0; } // out into the fields
    if keyboard.pressed(KeyCode::KeyS) { direction.z += 1.0; } // back to camp
    if keyboard.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
    if keyboard.pressed(KeyCode::KeyD) { direction.x += 1.0; }

    if direction == Vec3::ZERO {
        return;
    }
    transform.translation += direction.normalize() * MOVE_SPEED * time.delta_secs();

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
