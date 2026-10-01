#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::components::WallSegment;

const ROWS: &[char] = &['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J'];
const GRID_SPACING: f32 = 10.0;
const WALL_OFFSET_Z: f32 = 60.0; // push walls away from origin

pub struct WallPlugin;

impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_wall_grid)
            .add_systems(Update, update_wall_visuals);
    }
}

/// Spawns 100 wall-segment cubes in a 10×10 grid.
fn spawn_wall_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(8.0, 12.0, 1.0));

    for (row_idx, row_char) in ROWS.iter().enumerate() {
        for col in 0..10_i32 {
            let x = col as f32 * GRID_SPACING - 45.0;
            let z = WALL_OFFSET_Z + row_idx as f32 * 2.0;
            let segment_id = format!("{}-{}", row_char, col);

            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.2, 0.8, 0.2),
                    ..default()
                })),
                Transform::from_xyz(x, 6.0, z),
                WallSegment {
                    segment_id,
                    visual_damage_state: 0,
                    integrity: 1.0,
                },
                Name::new(format!("Wall-{}-{}", row_char, col)),
            ));
        }
    }
}

/// Updates wall material colour based on damage state.
fn update_wall_visuals(
    mut query: Query<(&WallSegment, &MeshMaterial3d<StandardMaterial>), Changed<WallSegment>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (wall, mat_handle) in &mut query {
        let color = match wall.visual_damage_state {
            0 => Color::srgb(0.2, 0.8, 0.2), // pristine — green
            1 => Color::srgb(0.8, 0.8, 0.1), // damaged  — yellow
            2 => Color::srgb(0.9, 0.4, 0.1), // critical — orange
            _ => Color::srgba(0.6, 0.1, 0.1, 0.4), // breached — dark red, semi-transparent
        };
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.base_color = color;
        }
    }
}
