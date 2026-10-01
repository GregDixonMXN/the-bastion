#![allow(dead_code, unused_imports)]

use spacetimedb::table;
use serde::{Deserialize, Serialize};

/// Mech unit — piloted by a player or left static when fuel runs out.
#[table(name = mech, public)]
#[derive(Clone, Debug)]
pub struct MechTable {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub owner_identity: spacetimedb::spacetimedb_lib::Identity,
    pub health: f32,
    pub max_health: f32,
    pub fuel: f32,
    pub max_fuel: f32,
    /// "LightScout" | "HeavyJuggernaut"
    pub mech_type: String,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    /// True when fuel == 0; mech is immobile.
    pub is_static: bool,
    /// JSON-encoded list of equipped weapon names.
    pub equipped_weapons: String,
}
