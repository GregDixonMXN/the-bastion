#![allow(dead_code)]

use bevy::prelude::*;

/// Enemy state mirrored from SpacetimeDB EnemyTable.
#[derive(Component, Debug, Clone)]
pub struct Enemy {
    pub db_id: u64,
    pub enemy_type: String,
    pub health: f32,
    pub max_health: f32,
}
