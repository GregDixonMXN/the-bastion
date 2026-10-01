#![allow(dead_code)]

use bevy::prelude::*;

/// Mech state mirrored from the SpacetimeDB MechTable.
#[derive(Component, Debug, Clone)]
pub struct Mech {
    pub db_id: u64,
    pub health: f32,
    pub max_health: f32,
    pub fuel: f32,
    pub max_fuel: f32,
    pub mech_type: String,
    pub is_static: bool,
}

/// Tag component — marks the mech belonging to the local player.
#[derive(Component, Debug, Clone, Default)]
pub struct LocalMech;
