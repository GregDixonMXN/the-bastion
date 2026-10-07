#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::components::Enemy;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_placeholder_enemy)
            .add_systems(Update, update_enemy_positions);
    }
}

/// Spawns one placeholder Gloomrat for visual testing.
fn spawn_placeholder_enemy(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.2, 0.8, 1.6))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.45, 0.3, 0.5),
            ..default()
        })),
        Transform::from_xyz(15.0, 0.4, 40.0),
        Enemy {
            db_id: 0,
            enemy_type: "gloomrat".to_string(),
            health: 60.0,
            max_health: 60.0,
        },
        Name::new("Enemy-Gloomrat"),
    ));
}

/// Updates enemy entity positions from DB state each tick.
/// TODO: replace with real SpacetimeDB subscription callbacks in Phase 2.
fn update_enemy_positions(query: Query<(Entity, &Enemy, &Transform)>) {
    // Stub — position sync will be driven by SpacetimeDB row-update callbacks.
    let _ = query;
}
