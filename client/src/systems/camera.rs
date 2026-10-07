#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::components::LocalCharacter;

/// Follow camera — smoothly tracks the local player's character.
pub fn follow_camera(
    character_query: Query<&Transform, (With<LocalCharacter>, Without<Camera3d>)>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(character_tf) = character_query.single() else { return };
    let Ok(mut cam_tf) = camera_query.single_mut() else { return };

    let target = character_tf.translation + Vec3::new(0.0, 12.0, -14.0);
    cam_tf.translation = cam_tf.translation.lerp(target, 0.1);
    cam_tf.look_at(character_tf.translation, Vec3::Y);
}
