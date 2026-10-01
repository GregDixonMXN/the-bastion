#![allow(dead_code)]

use bevy::prelude::*;

/// Marks the entity controlled by the local player.
#[derive(Component, Debug, Clone)]
pub struct LocalPlayer {
    pub identity: String,
    pub username: String,
    pub sentence_remaining: u64,
    pub is_rogue: bool,
}
