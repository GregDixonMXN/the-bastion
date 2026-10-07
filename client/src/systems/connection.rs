#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use spacetimedb_sdk::{DbContext, Table, TableWithPrimaryKey};
use crate::generated::{
    register_player_reducer::register_player, spawn_character_reducer::spawn_character,
    CharacterTableAccess, DbConnection, EnemyTableAccess, LootTableAccess,
};
use crate::resources::{GameState, NetEvent, NetState};

pub const SPACETIME_URL: &str = "ws://127.0.0.1:3000";
pub const DB_NAME: &str = "bastionlands";

/// Startup system — builds the connection, registers row callbacks, starts
/// the background pump, and moves to Connecting. The `on_connect` callback
/// registers the player and subscribes to all tables.
pub fn initiate_connection(mut net: ResMut<NetState>, mut next_state: ResMut<NextState<GameState>>) {
    let tx = net.sender();
    let tx_connect = tx.clone();

    let conn = match DbConnection::builder()
        .with_uri(SPACETIME_URL)
        .with_module_name(DB_NAME)
        .on_connect(move |conn, identity, _token| {
            log::info!("SpacetimeDB connected, identity={identity}");
            let _ = tx_connect.send(NetEvent::Connected(identity));
            if let Err(e) = conn.reducers.register_player("Wanderer".to_string()) {
                log::warn!("register_player send failed: {e}");
            }
            let tx_sub = tx_connect.clone();
            conn.subscription_builder()
                .on_applied(move |ctx| {
                    log::info!("Subscription applied — requesting Adventurer");
                    // Spawns unless one already lives; server-side no-op otherwise.
                    if let Err(e) = ctx.reducers.spawn_character("Adventurer".to_string()) {
                        log::warn!("spawn_character send failed: {e}");
                    }
                    let _ = &tx_sub;
                })
                .on_error(|_ctx, e| {
                    log::error!("Subscription error: {e}");
                })
                .subscribe([
                    "SELECT * FROM player",
                    "SELECT * FROM character",
                    "SELECT * FROM enemy",
                    "SELECT * FROM loot",
                ]);
        })
        .on_connect_error(|_ctx, e| {
            log::error!("SpacetimeDB connect error: {e}");
        })
        .on_disconnect(|_ctx, e| {
            log::warn!("SpacetimeDB disconnected: {e:?}");
        })
        .build()
    {
        Ok(conn) => conn,
        Err(e) => {
            log::error!("Could not build SpacetimeDB connection: {e}");
            return;
        }
    };

    // Row callbacks → bridge channel. Dropping the returned IDs is safe;
    // removal is explicit via `remove_on_*`, which we never call.
    {
        let txc = tx.clone();
        conn.db.character().on_insert(move |_ctx, row| {
            let _ = txc.send(NetEvent::Character(row.clone()));
        });
        let txc = tx.clone();
        conn.db.character().on_update(move |_ctx, _old, row| {
            let _ = txc.send(NetEvent::Character(row.clone()));
        });
        let txc = tx.clone();
        conn.db.character().on_delete(move |_ctx, row| {
            let _ = txc.send(NetEvent::CharacterGone(row.id));
        });
        let txc = tx.clone();
        conn.db.enemy().on_insert(move |_ctx, row| {
            let _ = txc.send(NetEvent::Enemy(row.clone()));
        });
        let txc = tx.clone();
        conn.db.enemy().on_update(move |_ctx, _old, row| {
            let _ = txc.send(NetEvent::Enemy(row.clone()));
        });
        let txc = tx.clone();
        conn.db.enemy().on_delete(move |_ctx, row| {
            let _ = txc.send(NetEvent::EnemyGone(row.id));
        });
        let txc = tx.clone();
        conn.db.loot().on_insert(move |_ctx, row| {
            let _ = txc.send(NetEvent::Loot(row.clone()));
        });
        let txc = tx.clone();
        conn.db.loot().on_update(move |_ctx, _old, row| {
            let _ = txc.send(NetEvent::Loot(row.clone()));
        });
        let txc = tx.clone();
        conn.db.loot().on_delete(move |_ctx, row| {
            let _ = txc.send(NetEvent::LootGone(row.id));
        });
    }

    conn.run_threaded();
    net.conn = Some(conn);
    next_state.set(GameState::Connecting);
    log::info!("SpacetimeDB pump running — connecting to {SPACETIME_URL} db={DB_NAME}");
}
