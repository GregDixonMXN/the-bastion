#![allow(dead_code, unused_imports)]

use spacetimedb::table;
use serde::{Deserialize, Serialize};

/// A Kaiju creature advancing on the Bastion walls.
#[table(name = kaiju, public)]
#[derive(Clone, Debug)]
pub struct KaijuTable {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    /// "Breaker" | "Parasite"
    pub kaiju_type: String,
    pub health: f32,
    pub max_health: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    /// Wall segment this Kaiju is targeting.
    pub target_wall_segment: String,
    /// JSON-encoded list of Identity strings that have attacked this Kaiju.
    pub aggro_list: String,
    pub speed: f32,
    pub damage: f32,
}
