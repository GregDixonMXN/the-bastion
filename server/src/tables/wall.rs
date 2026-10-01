#![allow(dead_code, unused_imports)]

use spacetimedb::table;
use serde::{Deserialize, Serialize};

/// A single wall segment in the defensive perimeter grid.
#[table(name = wall, public)]
#[derive(Clone, Debug)]
pub struct WallTable {
    /// Segment identifier, e.g. "A-4".
    #[primary_key]
    pub segment_id: String,
    pub health: f32,
    pub max_health: f32,
    /// Normalised 0.0–1.0 structural integrity.
    pub integrity: f32,
    /// 0 = pristine, 1 = damaged, 2 = critical, 3 = breached.
    pub visual_damage_state: u8,
    pub grid_x: i32,
    pub grid_z: i32,
}
