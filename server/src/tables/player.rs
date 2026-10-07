#![allow(dead_code, unused_imports)]

use spacetimedb::{spacetimedb_lib::Identity, table};
use serde::{Deserialize, Serialize};

/// Player account state. One account, one active character for now.
#[table(name = player, public)]
#[derive(Clone, Debug)]
pub struct PlayerTable {
    #[primary_key]
    pub identity: Identity,
    pub username: String,
    pub online_status: bool,
    pub current_character_id: Option<u64>,
}
