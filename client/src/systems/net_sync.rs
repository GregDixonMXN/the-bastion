#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::components::{CampProp, Character, Enemy, LocalCharacter, LootItem};
use crate::resources::{GameState, NetEvent, NetState};

/// Drains the SpacetimeDB event bridge each frame and mirrors rows into
/// entities. Server state is authoritative: stats always apply, transforms
/// apply to everything except our own predicted character.
pub fn drain_net_events(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut net: ResMut<NetState>,
    mut next_state: ResMut<NextState<GameState>>,
    characters: Query<(Entity, &Character)>,
    enemies: Query<(Entity, &Enemy)>,
    loot: Query<(Entity, &LootItem)>,
    local: Query<Entity, With<LocalCharacter>>,
) {
    let events: Vec<NetEvent> = {
        let rx = net.rx.lock().unwrap();
        rx.try_iter().collect()
    };
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
                        commands.entity(entity).insert(Transform::from_xyz(
                            row.pos_x,
                            1.1,
                            row.pos_z,
                        ));
                    }
                    if row.is_dead && is_ours {
                        log::info!("You died — press R at camp to respawn");
                    }
                } else {
                    let color = if is_ours {
                        Color::srgb(0.2, 0.55, 0.9)
                    } else {
                        Color::srgb(0.9, 0.55, 0.2)
                    };
                    let entity = commands
                        .spawn((
                            Mesh3d(meshes.add(Cuboid::new(1.0, 2.2, 1.0))),
                            MeshMaterial3d(materials.add(StandardMaterial {
                                base_color: color,
                                ..default()
                            })),
                            Transform::from_xyz(row.pos_x, 1.1, row.pos_z),
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
                        ))
                        .id();
                    if is_ours && local.is_empty() {
                        commands.entity(entity).insert(LocalCharacter);
                        net.own_character_id = Some(row.id);
                        log::info!("Local character bound: {} (id {})", row.name, row.id);
                    }
                }
                // Retire the startup placeholder once the real row lands.
                if is_ours {
                    for (entity, c) in &characters {
                        if c.db_id == 0 {
                            commands.entity(entity).despawn();
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
                if let Some((entity, _)) = enemies.iter().find(|(_, e)| e.db_id == row.id) {
                    commands.entity(entity).insert((
                        Transform::from_xyz(row.pos_x, 0.4, row.pos_z),
                        Enemy {
                            db_id: row.id,
                            enemy_type: row.enemy_type.clone(),
                            health: row.health,
                            max_health: row.max_health,
                        },
                    ));
                } else {
                    commands.spawn((
                        Mesh3d(meshes.add(Cuboid::new(1.2, 0.8, 1.6))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(0.45, 0.3, 0.5),
                            ..default()
                        })),
                        Transform::from_xyz(row.pos_x, 0.4, row.pos_z),
                        Enemy {
                            db_id: row.id,
                            enemy_type: row.enemy_type.clone(),
                            health: row.health,
                            max_health: row.max_health,
                        },
                        Name::new(format!("Enemy-{}", row.enemy_type)),
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
                    Transform::from_xyz(row.pos_x, 0.25, row.pos_z),
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
