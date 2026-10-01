#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::components::{LocalMech, Mech};

const MOVE_SPEED: f32 = 10.0;

/// Reads WASD input and translates the local mech entity.
/// A real implementation would call the SpacetimeDB `move_mech` reducer here.
pub fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Mech), With<LocalMech>>,
) {
    let Ok((mut transform, mech)) = query.single_mut() else { return };

    if mech.is_static {
        return; // out of fuel — cannot move
    }

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) { direction.z += 1.0; }  // toward walls
    if keyboard.pressed(KeyCode::KeyS) { direction.z -= 1.0; } // back to base
    if keyboard.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
    if keyboard.pressed(KeyCode::KeyD) { direction.x += 1.0; }

    if direction != Vec3::ZERO {
        let move_delta = direction.normalize() * MOVE_SPEED * time.delta_secs();
        transform.translation += move_delta;

        // TODO: call SpacetimeDB `move_mech` reducer with new position.
        log::debug!(
            "Local mech moved to ({:.1}, {:.1})",
            transform.translation.x,
            transform.translation.z
        );
    }
}
