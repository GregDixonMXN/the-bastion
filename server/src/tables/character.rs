#![allow(dead_code, unused_imports)]

use spacetimedb::table;
use serde::{Deserialize, Serialize};

/// Playable character. The only base class is "Adventurer" — the community
/// grows the roster by submitting spec-compliant characters (see
/// CHARACTER_SPEC.md). Stats live here, not on the player, so future
/// characters (including approved community ones) share one progression shape.
#[table(name = character, public)]
#[derive(Clone, Debug)]
pub struct CharacterTable {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub owner_identity: spacetimedb::spacetimedb_lib::Identity,
    pub name: String,
    /// Base class key, e.g. "adventurer". Community classes arrive later.
    pub class: String,
    pub level: u32,
    pub xp: u64,
    pub gold: u64,
    pub health: f32,
    pub max_health: f32,
    pub mana: f32,
    pub max_mana: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    /// Seconds of death timestamp behaviour handled client-side; true while
    /// awaiting respawn.
    pub is_dead: bool,
}
