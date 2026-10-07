#![allow(dead_code)]

use bevy::prelude::*;

/// Playable character state mirrored from the SpacetimeDB CharacterTable.
#[derive(Component, Debug, Clone)]
pub struct Character {
    pub db_id: u64,
    pub name: String,
    pub class: String,
    pub level: u32,
    pub xp: u64,
    pub gold: u64,
    pub health: f32,
    pub max_health: f32,
    pub mana: f32,
    pub max_mana: f32,
    pub is_dead: bool,
}

/// Tag component — marks the character belonging to the local player.
#[derive(Component, Debug, Clone, Default)]
pub struct LocalCharacter;
