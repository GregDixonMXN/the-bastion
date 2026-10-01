#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::components::{LocalMech, Mech};
use crate::systems::player_input;
use crate::systems::camera::follow_camera;

pub struct MechPlugin;

impl Plugin for MechPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player_mech)
            .add_systems(Update, (player_input::player_input, follow_camera, log_fuel));
    }
}

/// Spawns a placeholder cube representing the local player's mech.
fn spawn_player_mech(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 3.0, 2.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.6, 1.0),
            ..default()
        })),
        Transform::from_xyz(0.0, 1.5, 0.0),
        Mech {
            db_id: 0,
            health: 1000.0,
            max_health: 1000.0,
            fuel: 500.0,
            max_fuel: 500.0,
            mech_type: "LightScout".to_string(),
            is_static: false,
        },
        LocalMech,
        Name::new("PlayerMech"),
    ));
}

/// Stub system — logs fuel level each second (replace with HUD in Phase 2).
fn log_fuel(query: Query<&Mech, With<LocalMech>>, time: Res<Time>) {
    // Only log once per second to avoid spam.
    if (time.elapsed_secs() % 1.0) < time.delta_secs() {
        if let Ok(mech) = query.get_single() {
            log::debug!("Fuel: {:.0}/{:.0}", mech.fuel, mech.max_fuel);
        }
    }
}
