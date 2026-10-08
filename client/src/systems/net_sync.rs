#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::components::{Character, Enemy, LocalCharacter, LootItem};
use crate::plugins::terrain_plugin::terrain_height;
use crate::resources::{GameState, NetEvent, NetState};
use super::animation::NeedsClips;
use super::locomotion::Locomotion;

/// Drains the SpacetimeDB event bridge each frame and mirrors rows into
/// entities. Server state is authoritative: stats always apply, transforms
/// apply to everything except our own predicted character.
pub fn drain_net_events(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut net: ResMut<NetState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut characters: Query<(Entity, &Character)>,
    mut enemies: Query<(Entity, &Enemy)>,
    mut locos: Query<&mut Locomotion>,
    loot: Query<(Entity, &LootItem)>,
) {
    let events: Vec<NetEvent> = {
        let rx = net.rx.lock().unwrap();
        rx.try_iter().collect()
    };
    if !events.is_empty() {
        log::debug!("net drain: {} events", events.len());
    }
    for event in events {
        match event {
            NetEvent::Connected(identity) => {
                net.connected = true;
                net.identity = Some(identity);
                next_state.set(GameState::InGame);
                log::info!("Net synced — entering game");
            }
            NetEvent::Disconnected(reason) => {
                net.connected = false;
                log::warn!("Net disconnected: {reason}");
            }
            NetEvent::Character(row) => {
                let is_ours = net.identity.as_ref().is_some_and(|id| *id == row.owner_identity);
                if let Some((entity, _)) = characters.iter().find(|(_, c)| c.db_id == row.id) {
                    // Refresh stats; leave our own transform to prediction.
                    if let Ok(mut e) = commands.get_entity(entity) {
                        e.insert(Character {
                            db_id: row.id,
                            name: row.name.clone(),
                            class: row.class.clone(),
                            level: row.level,
                            xp: row.xp,
                            gold: row.gold,
                            health: row.health,
                            max_health: row.max_health,
                            mana: row.mana,
                            max_mana: row.max_mana,
                            is_dead: row.is_dead,
                        });
                    }
                    if !is_ours {
                        let ground = terrain_height(row.pos_x, row.pos_z);
                        commands.entity(entity).insert(Transform::from_xyz(
                            row.pos_x,
                            ground,
                            row.pos_z,
                        ));
                        if let Ok(mut l) = locos.get_mut(entity) {
                            l.base_y = ground;
                        }
                    }
                    if row.is_dead && is_ours {
                        log::info!("You died — press R at camp to respawn");
                    }
                } else {
                    let ground = terrain_height(row.pos_x, row.pos_z);
                    let pos = Vec3::new(row.pos_x, ground, row.pos_z);
                    let entity = commands
                        .spawn((
                            SceneRoot(
                                asset_server.load("models/characters/adventurer.glb#Scene0"),
                            ),
                            Transform::from_xyz(
                                row.pos_x,
                                ground,
                                row.pos_z,
                            ),
                            Character {
                                db_id: row.id,
                                name: row.name.clone(),
                                class: row.class.clone(),
                                level: row.level,
                                xp: row.xp,
                                gold: row.gold,
                                health: row.health,
                                max_health: row.max_health,
                                mana: row.mana,
                                max_mana: row.max_mana,
                                is_dead: row.is_dead,
                            },
                            Name::new(format!("Character-{}", row.name)),
                            Locomotion::new(ground, pos, 0.10),
                            NeedsClips,
                        ))
                        .id();
                    if is_ours {
                        // Exactly one local tag per session: newest owned row
                        // wins (batch delivery would otherwise tag them all,
                        // breaking every `.single()` downstream).
                        let take = match net.own_character_id {
                            None => true,
                            Some(cur) => row.id > cur,
                        };
                        if take {
                            if let Some(cur) = net.own_character_id {
                                for (tagged, c) in &characters {
                                    if c.db_id == cur {
                                        commands.entity(tagged).remove::<LocalCharacter>();
                                    }
                                }
                            }
                            commands.entity(entity).insert(LocalCharacter);
                            net.own_character_id = Some(row.id);
                            log::info!("Local character bound: {} (id {})", row.name, row.id);
                        }
                    }
                }
            }
            NetEvent::CharacterGone(id) => {
                for (entity, c) in &characters {
                    if c.db_id == id {
                        commands.entity(entity).despawn();
                    }
                }
                if net.own_character_id == Some(id) {
                    net.own_character_id = None;
                }
            }
            NetEvent::Enemy(row) => {
                let ground = terrain_height(row.pos_x, row.pos_z);
                // Thornwolf uses the fox stand-in scaled up until its own
                // model lands (see ASSETS.md).
                let scale = if row.enemy_type == "thornwolf" { 1.6 } else { 1.0 };
                if let Some((entity, _)) = enemies.iter().find(|(_, e)| e.db_id == row.id) {
                    let base = ground + 0.4 * scale;
                    commands.entity(entity).insert((
                        Transform::from_xyz(row.pos_x, base, row.pos_z)
                            .with_scale(Vec3::splat(scale)),
                        Enemy {
                            db_id: row.id,
                            enemy_type: row.enemy_type.clone(),
                            health: row.health,
                            max_health: row.max_health,
                        },
                    ));
                    if let Ok(mut l) = locos.get_mut(entity) {
                        l.base_y = base;
                    }
                } else {
                    let base = ground + 0.4 * scale;
                    let pos = Vec3::new(row.pos_x, base, row.pos_z);
                    commands.spawn((
                        SceneRoot(asset_server.load("models/enemies/gloomrat.glb#Scene0")),
                        Transform::from_xyz(row.pos_x, base, row.pos_z)
                            .with_scale(Vec3::splat(scale)),
                        Enemy {
                            db_id: row.id,
                            enemy_type: row.enemy_type.clone(),
                            health: row.health,
                            max_health: row.max_health,
                        },
                        Name::new(format!("Enemy-{}", row.enemy_type)),
                        Locomotion::new(base, pos, 0.15),
                        NeedsClips,
                    ));
                }
            }
            NetEvent::EnemyGone(id) => {
                for (entity, e) in &enemies {
                    if e.db_id == id {
                        commands.entity(entity).despawn();
                    }
                }
            }
            NetEvent::Loot(row) => {
                if row.is_collected {
                    for (entity, l) in &loot {
                        if l.db_id == row.id {
                            commands.entity(entity).despawn();
                        }
                    }
                    continue;
                }
                if loot.iter().any(|(_, l)| l.db_id == row.id) {
                    continue;
                }
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 0.85, 0.2),
                        emissive: Color::srgb(0.6, 0.45, 0.05).into(),
                        ..default()
                    })),
                    Transform::from_xyz(row.pos_x, terrain_height(row.pos_x, row.pos_z) + 0.25, row.pos_z),
                    LootItem {
                        db_id: row.id,
                        loot_type: row.loot_type.clone(),
                        value: row.value,
                    },
                    Name::new("Loot"),
                ));
            }
            NetEvent::LootGone(id) => {
                for (entity, l) in &loot {
                    if l.db_id == id {
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    }
}
