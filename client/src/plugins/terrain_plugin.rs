#![allow(dead_code)]

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

/// Deterministic value-noise field shared by terrain generation and entity
/// placement, so feet always meet the ground. No dependencies, fixed seed.
fn hash2(x: i32, z: i32) -> f32 {
    let mut h = (x.wrapping_mul(374761393).wrapping_add(z.wrapping_mul(668265263))) as u32;
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    ((h ^ (h >> 16)) as f32) / (u32::MAX as f32)
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn value_noise(x: f32, z: f32) -> f32 {
    let xi = x.floor() as i32;
    let zi = z.floor() as i32;
    let xf = x - xi as f32;
    let zf = z - zi as f32;
    let a = hash2(xi, zi);
    let b = hash2(xi + 1, zi);
    let c = hash2(xi, zi + 1);
    let d = hash2(xi + 1, zi + 1);
    let u = smooth(xf);
    let v = smooth(zf);
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

/// Rolling starter-field height in meters. Gentle near camp (flat spawn),
/// hillier further out.
pub fn terrain_height(x: f32, z: f32) -> f32 {
    let dist = (x * x + z * z).sqrt();
    let flat = (dist / 25.0).clamp(0.0, 1.0); // pancake-flat inside camp
    let hills = value_noise(x * 0.02, z * 0.02) * 8.0
        + value_noise(x * 0.06 + 100.0, z * 0.06) * 2.5
        - 5.0;
    let shore = -((dist - 95.0) / 25.0).clamp(0.0, 1.0) * 4.0; // dip to water
    hills * flat + shore
}

fn color_for_height(h: f32) -> [f32; 4] {
    if h < 0.15 {
        [0.76, 0.70, 0.45, 1.0] // sand
    } else if h < 3.5 {
        [0.25, 0.45, 0.22, 1.0] // meadow
    } else if h < 6.0 {
        [0.20, 0.35, 0.20, 1.0] // deep field
    } else {
        [0.45, 0.42, 0.38, 1.0] // rock
    }
}

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_terrain, spawn_water, scatter_nature));
    }
}

const SIZE: f32 = 240.0;
const SEGS: u32 = 120;

fn spawn_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let verts_per_row = SEGS + 1;
    let mut positions = Vec::with_capacity((verts_per_row * verts_per_row) as usize);
    let mut normals = Vec::with_capacity((verts_per_row * verts_per_row) as usize);
    let mut colors = Vec::with_capacity((verts_per_row * verts_per_row) as usize);
    let mut indices = Vec::with_capacity((SEGS * SEGS * 6) as usize);

    for iz in 0..=SEGS {
        for ix in 0..=SEGS {
            let x = (ix as f32 / SEGS as f32 - 0.5) * SIZE;
            let z = (iz as f32 / SEGS as f32 - 0.5) * SIZE;
            let h = terrain_height(x, z);
            positions.push([x, h, z]);
            normals.push([0.0, 1.0, 0.0]);
            colors.push(color_for_height(h));
        }
    }
    for iz in 0..SEGS {
        for ix in 0..SEGS {
            let a = iz * verts_per_row + ix;
            let b = a + 1;
            let c = a + verts_per_row;
            let d = c + 1;
            indices.extend([a, c, b, b, c, d]);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, Default::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    // Flat shading reads better at this scale; smooth normals stay up.
    mesh.duplicate_vertices();
    mesh.compute_flat_normals();

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.95,
            ..default()
        })),
        Name::new("Terrain"),
    ));
}

fn spawn_water(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(SIZE, SIZE))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.15, 0.35, 0.55, 0.75),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.2,
            metallic: 0.1,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.3, 0.0),
        Name::new("Water"),
    ));
}

/// Deterministic LCG for scatter placement (fixed world, every run).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) as f32) / (u32::MAX as f32)
    }
}

fn scatter_nature(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = Rng(0xB4571A9D);
    let trunk_mesh = meshes.add(Cylinder::new(0.3, 2.0));
    let leaf_mesh = meshes.add(Cone::new(1.8, 4.0));
    let rock_mesh = meshes.add(Sphere::new(1.0).mesh().ico(1).unwrap());
    let trunk_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.27, 0.15),
        perceptual_roughness: 0.9,
        ..default()
    });
    let leaf_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.16, 0.38, 0.18),
        perceptual_roughness: 0.9,
        ..default()
    });
    let rock_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.52),
        perceptual_roughness: 0.95,
        ..default()
    });

    let mut placed = 0;
    let mut guard = 0;
    while placed < 70 && guard < 2000 {
        guard += 1;
        let x = (rng.next() * 2.0 - 1.0) * 110.0;
        let z = (rng.next() * 2.0 - 1.0) * 110.0;
        if (x * x + z * z).sqrt() < 14.0 {
            continue; // keep camp clearing open
        }
        let h = terrain_height(x, z);
        if h < 0.4 {
            continue; // not in the water
        }
        let s = 0.8 + rng.next() * 0.9;
        commands.spawn((
            Mesh3d(trunk_mesh.clone()),
            MeshMaterial3d(trunk_mat.clone()),
            Transform::from_xyz(x, h + 1.0 * s, z).with_scale(Vec3::splat(s)),
            Name::new("TreeTrunk"),
        ));
        commands.spawn((
            Mesh3d(leaf_mesh.clone()),
            MeshMaterial3d(leaf_mat.clone()),
            Transform::from_xyz(x, h + (2.0 + 2.0) * s, z).with_scale(Vec3::splat(s)),
            Name::new("TreeTop"),
        ));
        placed += 1;
    }

    for _ in 0..35 {
        let x = (rng.next() * 2.0 - 1.0) * 110.0;
        let z = (rng.next() * 2.0 - 1.0) * 110.0;
        if (x * x + z * z).sqrt() < 12.0 {
            continue;
        }
        let h = terrain_height(x, z);
        if h < 0.3 {
            continue;
        }
        let s = 0.4 + rng.next() * 1.2;
        commands.spawn((
            Mesh3d(rock_mesh.clone()),
            MeshMaterial3d(rock_mat.clone()),
            Transform::from_xyz(x, h + 0.3 * s, z)
                .with_scale(Vec3::new(s, s * 0.7, s))
                .with_rotation(Quat::from_rotation_y(rng.next() * 6.28)),
            Name::new("Rock"),
        ));
    }
}
