#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::components::{Character, LocalCharacter};
use crate::systems::player_input;
use crate::systems::camera::follow_camera;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player_character)
            .add_systems(Update, (player_input::player_input, follow_camera, log_health));
    }
}

/// Spawns a placeholder capsule representing the local Adventurer.
fn spawn_player_character(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 2.2, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.55, 0.9),
            ..default()
        })),
        Transform::from_xyz(0.0, 1.1, 0.0),
        Character {
            db_id: 0,
            name: "Adventurer".to_string(),
            class: "adventurer".to_string(),
            level: 1,
            health: 100.0,
            max_health: 100.0,
            mana: 50.0,
            max_mana: 50.0,
            is_dead: false,
        },
        LocalCharacter,
        Name::new("PlayerCharacter"),
    ));
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
