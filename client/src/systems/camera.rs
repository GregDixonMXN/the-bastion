#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::components::LocalMech;

/// Follow camera — smoothly tracks the local player's mech.
pub fn follow_camera(
    mech_query: Query<&Transform, (With<LocalMech>, Without<Camera3d>)>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(mech_tf) = mech_query.single() else { return };
    let Ok(mut cam_tf) = camera_query.single_mut() else { return };

    let target = mech_tf.translation + Vec3::new(0.0, 20.0, -25.0);
    cam_tf.translation = cam_tf.translation.lerp(target, 0.1);
    cam_tf.look_at(mech_tf.translation, Vec3::Y);
}
