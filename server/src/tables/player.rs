#![allow(dead_code, unused_imports)]

use spacetimedb::{spacetimedb_lib::Identity, table};
use serde::{Deserialize, Serialize};

/// Player state persisted in SpacetimeDB.
#[table(name = player, public)]
#[derive(Clone, Debug)]
pub struct PlayerTable {
    #[primary_key]
    pub identity: Identity,
    pub username: String,
    /// XP/currency — hours of sentence served/reduced.
    pub sentence_remaining: u64,
    pub current_mech_id: Option<u64>,
    pub online_status: bool,
    /// Bounty flag — triggers automated turret targeting.
    pub is_rogue: bool,
}
