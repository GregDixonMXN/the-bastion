#![allow(dead_code)]

use bevy::prelude::*;

/// Top-level game connection state.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Disconnected,
    Connecting,
    InGame,
}
