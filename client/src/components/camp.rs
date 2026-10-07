#![allow(dead_code)]

use bevy::prelude::*;

/// Static camp dressing (gate, tents). No health, no sync — pure scenery
/// until camp services (respawn point, vendors) land in a later phase.
#[derive(Component, Debug, Clone)]
pub struct CampProp {
    pub prop_id: String,
    pub kind: String,
}
