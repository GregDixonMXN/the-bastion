#![allow(dead_code, unused_imports)]

use spacetimedb::table;
use serde::{Deserialize, Serialize};

/// Loot item dropped on the battlefield.
#[table(name = loot, public)]
#[derive(Clone, Debug)]
pub struct LootTable {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    /// "KaijuCore" | "ScrapMetal" | "FuelCell" | "MechPart"
    pub loot_type: String,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    /// Sentence-reduction value of this item.
    pub value: u64,
    pub dropped_by_kaiju_id: Option<u64>,
    pub is_collected: bool,
}
