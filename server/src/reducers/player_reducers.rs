#![allow(dead_code, unused_imports, unused_variables)]

use spacetimedb::{reducer, spacetimedb_lib::Identity, ReducerContext, Table};

// Bring the generated table-accessor traits into scope.
use crate::tables::player::player;
use crate::tables::player::PlayerTable;

/// Called on first connect — creates a player row if one does not exist.
#[reducer]
pub fn register_player(ctx: &ReducerContext, username: String) {
    let identity = ctx.sender;
    if ctx.db.player().identity().find(&identity).is_some() {
        log::info!("Player {:?} already registered", identity);
        return;
    }
    ctx.db.player().insert(PlayerTable {
        identity,
        username,
        sentence_remaining: 3600, // start with 1 hour on sentence
        current_mech_id: None,
        online_status: true,
        is_rogue: false,
    });
    log::info!("Registered new player {:?}", identity);
}

/// Awards sentence reduction to a player.
#[reducer]
pub fn update_sentence(ctx: &ReducerContext, identity: Identity, amount: u64) {
    use crate::tables::player::player;
    if let Some(mut p) = ctx.db.player().identity().find(&identity) {
        p.sentence_remaining = p.sentence_remaining.saturating_sub(amount);
        ctx.db.player().identity().update(p);
    }
}

/// Flags a player as rogue — automated turrets will target them.
#[reducer]
pub fn mark_rogue(ctx: &ReducerContext, identity: Identity) {
    use crate::tables::player::player;
    if let Some(mut p) = ctx.db.player().identity().find(&identity) {
        p.is_rogue = true;
        ctx.db.player().identity().update(p);
        log::warn!("Player {:?} marked ROGUE", identity);
    }
}

/// Sets a player offline and persists mech state.
#[reducer]
pub fn player_logout(ctx: &ReducerContext, identity: Identity) {
    use crate::tables::player::player;
    if let Some(mut p) = ctx.db.player().identity().find(&identity) {
        p.online_status = false;
        ctx.db.player().identity().update(p);
        log::info!("Player {:?} logged out", identity);
    }
}
