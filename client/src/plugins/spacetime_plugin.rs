#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::resources::{DbHandle, GameState};
use crate::systems::connection::{DB_NAME, SPACETIME_URL};

pub struct SpacetimePlugin;

impl Plugin for SpacetimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_spacetime_connection);
        // TODO: register on_connect / on_disconnect callbacks via spacetimedb-sdk
        // once generated client types are available (run `spacetime generate`).
    }
}

fn setup_spacetime_connection(
    mut db_handle: ResMut<DbHandle>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    log::info!(
        "SpacetimePlugin: attempting connection to {} db={}",
        SPACETIME_URL,
        DB_NAME
    );
    next_state.set(GameState::Connecting);

    // ── Phase 2: replace this stub with real SDK connection ──────────────────
    // let conn = spacetimedb_sdk::DbConnection::builder()
    //     .with_uri(SPACETIME_URL)
    //     .with_module_name(DB_NAME)
    //     .on_connect(|conn, identity, token| {
    //         log::info!("Connected! identity = {:?}", identity);
    //         conn.reducers.register_player("Pilot".to_string());
    //     })
    //     .build()
    //     .expect("Failed to connect to SpacetimeDB");
    //
    // Subscribe to all tables:
    // conn.subscription_builder()
    //     .subscribe(["SELECT * FROM player", "SELECT * FROM mech",
    //                 "SELECT * FROM wall",  "SELECT * FROM kaiju",
    //                 "SELECT * FROM loot"]);
    // ─────────────────────────────────────────────────────────────────────────

    db_handle.subscribed = false;
    log::info!("SpacetimePlugin stub ready — awaiting Phase 2 SDK integration");
}
