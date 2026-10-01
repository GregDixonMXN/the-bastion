#![allow(dead_code)]

use bevy::prelude::*;

/// Kaiju state mirrored from SpacetimeDB KaijuTable.
#[derive(Component, Debug, Clone)]
pub struct Kaiju {
    pub db_id: u64,
    pub kaiju_type: String,
    pub health: f32,
    pub max_health: f32,
}
