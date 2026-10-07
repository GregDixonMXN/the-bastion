#![allow(dead_code, unused_imports, unused_variables)]

use spacetimedb::{reducer, spacetimedb_lib::Identity, ReducerContext, Table};

use crate::tables::player::player;
use crate::tables::player::PlayerTable;

/// Called on first connect — creates a player row if one does not exist.
/// Identity always comes from the connection, never from a parameter.
#[reducer]
pub fn register_player(ctx: &ReducerContext, username: String) {
    let identity = ctx.sender;
    if ctx.db.player().identity().find(&identity).is_some() {
        return;
    }
    let clean = username.trim().chars().take(24).collect::<String>();
    ctx.db.player().insert(PlayerTable {
        identity,
        username: if clean.is_empty() { "Wanderer".to_string() } else { clean },
        online_status: true,
        current_character_id: None,
    });
}

/// Marks the sender offline. No identity parameter — you can only log
/// yourself out.
#[reducer]
pub fn player_logout(ctx: &ReducerContext) {
    if let Some(mut p) = ctx.db.player().identity().find(&ctx.sender) {
        p.online_status = false;
        ctx.db.player().identity().update(p);
    }
}
