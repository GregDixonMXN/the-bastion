#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::components::{Character, LocalCharacter};

const MOVE_SPEED: f32 = 6.0;

/// Reads WASD input and translates the local character.
/// A real implementation would call the SpacetimeDB `move_character` reducer here.
pub fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Character), With<LocalCharacter>>,
) {
    let Ok((mut transform, character)) = query.single_mut() else { return };

    if character.is_dead {
        return; // dead — respawn at camp first
    }

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) { direction.z -= 1.0; } // out into the fields
    if keyboard.pressed(KeyCode::KeyS) { direction.z += 1.0; } // back to camp
    if keyboard.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
    if keyboard.pressed(KeyCode::KeyD) { direction.x += 1.0; }

    if direction != Vec3::ZERO {
        let move_delta = direction.normalize() * MOVE_SPEED * time.delta_secs();
        transform.translation += move_delta;

        // TODO: call SpacetimeDB `move_character` reducer with new position.
        log::debug!(
            "Local character moved to ({:.1}, {:.1})",
            transform.translation.x,
            transform.translation.z
        );
    }
}
