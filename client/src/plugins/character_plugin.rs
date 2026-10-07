#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::components::{Character, LocalCharacter};
use crate::systems::combat_input;
use crate::systems::locomotion::animate_locomotion;
use crate::systems::player_input;
use crate::systems::camera::orbit_camera;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        // No startup placeholder: the sync system spawns our real character
        // from its database row within a second of connecting. (A placeholder
        // carrying LocalCharacter would steal the local tag — see net_sync.)
        app.add_systems(Update, (player_input::player_input, combat_input, orbit_camera, animate_locomotion, log_health));
    }
}

/// Stub system — logs health level each second (replace with HUD in Phase 2).
fn log_health(query: Query<&Character, With<LocalCharacter>>, time: Res<Time>) {
    // Only log once per second to avoid spam.
    if (time.elapsed_secs() % 1.0) < time.delta_secs() {
        if let Ok(character) = query.get_single() {
            log::debug!("HP: {:.0}/{:.0}", character.health, character.max_health);
        }
    }
}
