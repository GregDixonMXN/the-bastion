#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::components::LocalCharacter;

/// Which look a spawned scene gets. Inserted alongside `NeedsClips`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleAs {
    Warrior,
    Fox,
}

/// Shared look materials. One handle each — the pulse system animates the
/// warrior emissive once and every warrior glows together.
#[derive(Resource)]
struct StyleMats {
    warrior: Handle<StandardMaterial>,
    fox: Handle<StandardMaterial>,
}

#[derive(Component)]
struct FollowLight;

#[derive(Resource)]
struct Pulse {
    t: f32,
}

fn make_style_mats(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.insert_resource(StyleMats {
        // Dark steel armor with a cyan rune glow. No textures on the mesh
        // yet, so presence comes from metal + emissive + lighting.
        warrior: materials.add(StandardMaterial {
            base_color: Color::srgb(0.05, 0.06, 0.09),
            metallic: 0.85,
            perceptual_roughness: 0.35,
            emissive: Color::srgb(0.1, 0.5, 0.6).into(),
            ..default()
        }),
        // Gloom fox: near-black fur with a faint violet menace.
        fox: materials.add(StandardMaterial {
            base_color: Color::srgb(0.07, 0.05, 0.10),
            metallic: 0.0,
            perceptual_roughness: 0.9,
            emissive: Color::srgb(0.15, 0.05, 0.25).into(),
            ..default()
        }),
    });
    commands.insert_resource(Pulse { t: 0.0 });
}

/// Rim + follow lighting. A cool back light carves silhouettes; a small
/// cyan lantern rides the local hero.
fn style_lights(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.55, 0.75, 1.0),
            illuminance: 6_000.0,
            shadows_enabled: false,
            ..default()
        },
        // Low from behind-left: rims shoulders and blades.
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.4, -2.2, 0.0)),
        Name::new("RimLight"),
    ));
    commands.spawn((
        PointLight {
            color: Color::srgb(0.3, 0.8, 0.9),
            intensity: 40.0,
            range: 12.0,
            ..default()
        },
        Transform::from_xyz(0.0, 3.0, 0.0),
        FollowLight,
        Name::new("HeroLantern"),
    ));
}

/// Walk scene trees depth-first, swapping every StandardMaterial for the
/// styled one. Retries each frame until at least one swap lands (scenes
/// stream in async).
fn style_spawns(
    mut commands: Commands,
    mats: Res<StyleMats>,
    roots: Query<(Entity, &StyleAs)>,
    children: Query<&Children>,
    mesh_mats: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    for (root, style) in &roots {
        let want = match style {
            StyleAs::Warrior => &mats.warrior,
            StyleAs::Fox => &mats.fox,
        };
        let mut stack: Vec<Entity> =
            children.get(root).map(|c| c.to_vec()).unwrap_or_default();
        let mut swapped = 0;
        while let Some(entity) = stack.pop() {
            if mesh_mats.contains(entity) {
                commands.entity(entity).insert(MeshMaterial3d(want.clone()));
                swapped += 1;
            }
            if let Ok(kids) = children.get(entity) {
                stack.extend(kids.to_vec());
            }
        }
        if swapped > 0 {
            commands.entity(root).remove::<StyleAs>();
        }
    }
}

fn follow_light(
    hero: Query<&Transform, With<LocalCharacter>>,
    mut lantern: Query<&mut Transform, (With<FollowLight>, Without<LocalCharacter>)>,
) {
    let Ok(hips) = hero.single() else { return };
    let Ok(mut lamp) = lantern.single_mut() else { return };
    lamp.translation = hips.translation + Vec3::new(1.5, 3.0, -1.5);
}

fn pulse_runes(time: Res<Time>, mut pulse: ResMut<Pulse>, mut mats: ResMut<Assets<StandardMaterial>>, handles: Res<StyleMats>) {
    pulse.t += time.delta_secs();
    if let Some(mat) = mats.get_mut(&handles.warrior) {
        let glow = 0.55 + 0.45 * (pulse.t * 2.2).sin();
        mat.emissive = Color::srgb(0.10 * glow + 0.05, 0.50 * glow + 0.10, 0.60 * glow + 0.10).into();
    }
}

pub struct StylePlugin;

impl Plugin for StylePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (make_style_mats, style_lights))
            .add_systems(Update, (style_spawns, follow_light, pulse_runes));
    }
}
