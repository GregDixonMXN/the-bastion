#![allow(dead_code, unused_imports)]

mod components;
mod generated;
mod plugins;
mod resources;
mod systems;

use bevy::prelude::*;
use plugins::{CampPlugin, CharacterPlugin, EnemyPlugin, SpacetimePlugin, TerrainPlugin, UiPlugin};
use resources::GameState;
use systems::animation::AnimationPlugin;
use systems::camera::{FacingCam, OrbitCam};

fn main() {
    // Logging is owned by Bevy's LogPlugin (RUST_LOG respected).
    // Do NOT call env_logger::init() here — double init kills all log output.
    App::new()
        // ── Core ──────────────────────────────────────────────────────────────
        .insert_resource(ClearColor(Color::srgb(0.05, 0.08, 0.12)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "The Bastion".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        // ── State ─────────────────────────────────────────────────────────────
        .init_state::<GameState>()
        .insert_resource(OrbitCam::default())
        .insert_resource(FacingCam::default())
        // ── Game Plugins ──────────────────────────────────────────────────────
        .add_plugins((
            SpacetimePlugin,
            TerrainPlugin,
            AnimationPlugin,
            CharacterPlugin,
            CampPlugin,
            EnemyPlugin,
            UiPlugin,
        ))
        // ── World Setup ───────────────────────────────────────────────────────
        .add_systems(Startup, (setup_world,))
        .run();
}

/// Spawns camera, ground plane, and ambient lighting.
fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.05, 0.08, 0.12)),
            ..default()
        },
        Transform::from_xyz(0.0, 15.0, -20.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("MainCamera"),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 400.0,
        ..default()
    });

    // Directional sun
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.5, 0.0)),
        Name::new("Sun"),
    ));

    // Ground plane — flat starter disc under camp; the terrain plugin owns
    // the real landscape.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(30.0, 30.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.42, 0.22),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.02, 0.0),
        Name::new("CampGround"),
    ));
}
