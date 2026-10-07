#![allow(dead_code, unused_imports)]

use spacetimedb::table;
use serde::{Deserialize, Serialize};

/// A hostile creature in the starter lands. Enemy roster starts with one
/// entry ("Gloomrat") and grows with the community — same spec pipeline as
/// player characters, minus approval for hostile use.
#[table(name = enemy, public)]
#[derive(Clone, Debug)]
pub struct EnemyTable {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    /// Enemy type key, e.g. "gloomrat".
    pub enemy_type: String,
    pub health: f32,
    pub max_health: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    /// Character this enemy is currently pursuing, if any.
    pub target_character: Option<u64>,
    /// Where this enemy lives. Strays past the leash walk home.
    pub home_x: f32,
    pub home_z: f32,
    pub speed: f32,
    pub damage: f32,
    /// XP awarded to the killer.
    pub xp_value: u64,
    /// Gold dropped as loot on death.
    pub gold_value: u64,
}
