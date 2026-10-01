#![allow(dead_code)]

use bevy::prelude::*;

/// Wall segment mirrored from SpacetimeDB WallTable.
#[derive(Component, Debug, Clone)]
pub struct WallSegment {
    pub segment_id: String,
    pub visual_damage_state: u8,
    pub integrity: f32,
}
