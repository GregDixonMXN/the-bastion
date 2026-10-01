#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::resources::{DbHandle, GameState};

pub const SPACETIME_URL: &str = "ws://localhost:3000";
pub const DB_NAME: &str = "the-bastion";

/// One-shot startup system — initiates the SpacetimeDB connection.
/// Full SDK integration is wired in the SpacetimePlugin.
pub fn initiate_connection(mut db_handle: ResMut<DbHandle>, mut next_state: ResMut<NextState<GameState>>) {
    log::info!("Connecting to SpacetimeDB at {} (db: {})", SPACETIME_URL, DB_NAME);
    next_state.set(GameState::Connecting);
    // Actual connection is handled by SpacetimePlugin callbacks.
    let _ = db_handle.as_mut();
}
