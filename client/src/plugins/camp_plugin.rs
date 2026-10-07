#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::components::CampProp;
use super::terrain_plugin::terrain_height;

pub struct CampPlugin;

impl Plugin for CampPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_starter_camp);
    }
}

/// Starter camp dressing: a gate arch at spawn plus a ring of tents.
/// Pure scenery — no health, no sync.
fn spawn_starter_camp(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let wood = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.3, 0.18),
        perceptual_roughness: 0.9,
        ..default()
    });
    let canvas = materials.add(StandardMaterial {
        base_color: Color::srgb(0.75, 0.68, 0.5),
        perceptual_roughness: 0.9,
        ..default()
    });

    // Gate posts at the spawn point.
    for x in [-3.0, 3.0] {
        let h = terrain_height(x, 0.0);
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.8, 6.0, 0.8))),
            MeshMaterial3d(wood.clone()),
            Transform::from_xyz(x, h + 3.0, 0.0),
            CampProp {
                prop_id: format!("gate-post-{x}"),
                kind: "gate".to_string(),
            },
            Name::new("CampGatePost"),
        ));
    }
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(7.6, 0.8, 0.8))),
        MeshMaterial3d(wood.clone()),
        Transform::from_xyz(0.0, terrain_height(0.0, 0.0) + 6.2, 0.0),
        CampProp {
            prop_id: "gate-beam".to_string(),
            kind: "gate".to_string(),
        },
        Name::new("CampGateBeam"),
    ));

    // Ring of four tents behind the gate.
    for (i, (x, z)) in [(-10.0, -8.0), (10.0, -8.0), (-12.0, 4.0), (12.0, 4.0)]
        .iter()
        .enumerate()
    {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(4.0, 3.0, 4.0))),
            MeshMaterial3d(canvas.clone()),
            Transform::from_xyz(*x, terrain_height(*x, *z) + 1.5, *z),
            CampProp {
                prop_id: format!("tent-{i}"),
                kind: "tent".to_string(),
            },
            Name::new(format!("Tent-{i}")),
        ));
    }
}
