#![allow(dead_code)]

use bevy::prelude::*;

/// Loot pick-up mirrored from SpacetimeDB LootTable.
#[derive(Component, Debug, Clone)]
pub struct LootItem {
    pub db_id: u64,
    pub loot_type: String,
    pub value: u64,
}
