#![allow(dead_code, unused_imports)]

use spacetimedb::table;
use serde::{Deserialize, Serialize};

/// A fixed EQ-style spawn point. One enemy at a time; when it dies, the
/// point ‘cools down’ for `respawn_secs`, then pops a fresh one. No waves,
/// no director — just the field, repopulating on its own rhythm.
#[table(name = spawn_point, public)]
#[derive(Clone, Debug)]
pub struct SpawnPointTable {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub enemy_type: String,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    /// Seconds between death and repop.
    pub respawn_secs: u64,
    /// Enemy currently holding this point, if it still lives.
    pub alive_enemy_id: Option<u64>,
    /// AI ticks remaining until repop once the point is empty.
    pub cooldown_ticks: u64,
}
