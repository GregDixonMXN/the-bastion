#![allow(dead_code)]

use bevy::prelude::*;

/// Holds connection metadata for the active SpacetimeDB session.
#[derive(Resource, Debug, Default)]
pub struct DbHandle {
    /// The local player's identity string (hex encoded).
    pub identity: Option<String>,
    /// True once the initial subscription sync is complete.
    pub subscribed: bool,
}
