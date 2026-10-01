#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::components::Kaiju;

pub struct KaijuPlugin;

impl Plugin for KaijuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_placeholder_kaiju)
            .add_systems(Update, update_kaiju_positions);
    }
}

/// Spawns one placeholder Kaiju cube for visual testing.
fn spawn_placeholder_kaiju(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(4.0, 6.0, 4.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.1, 0.1),
            ..default()
        })),
        Transform::from_xyz(20.0, 3.0, 100.0),
        Kaiju {
            db_id: 0,
            kaiju_type: "Breaker".to_string(),
            health: 2000.0,
            max_health: 2000.0,
        },
        Name::new("Kaiju-Breaker"),
    ));
}

/// Updates Kaiju entity positions from DB state each tick.
/// TODO: replace with real SpacetimeDB subscription callbacks in Phase 2.
fn update_kaiju_positions(query: Query<(Entity, &Kaiju, &Transform)>) {
    // Stub — position sync will be driven by SpacetimeDB row-update callbacks.
    let _ = query;
}
