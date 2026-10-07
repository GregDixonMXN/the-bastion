#![allow(dead_code, unused_imports)]

use spacetimedb::table;
use serde::{Deserialize, Serialize};

/// Loot waiting on the ground. `value` is gold credited on pickup.
#[table(name = loot, public)]
#[derive(Clone, Debug)]
pub struct LootTable {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    /// "GoldCache" | "Pelt" | "Herb" | "Trinket" — all convert to gold for now.
    pub loot_type: String,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub value: u64,
    pub dropped_by_enemy_id: Option<u64>,
    pub is_collected: bool,
}
