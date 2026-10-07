#![allow(dead_code)]

use bevy::prelude::*;
use spacetimedb_sdk::Identity;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use std::time::Instant;

use crate::generated::{CharacterTable, DbConnection, EnemyTable, LootTable};

/// Events ferried from SpacetimeDB callbacks (background pump thread) to
/// Bevy systems (main thread).
pub enum NetEvent {
    Connected(Identity),
    Disconnected(String),
    Character(CharacterTable),
    CharacterGone(u64),
    Enemy(EnemyTable),
    EnemyGone(u64),
    Loot(LootTable),
    LootGone(u64),
}

/// Shared network state: the live connection plus the event bridge.
#[derive(Resource)]
pub struct NetState {
    pub conn: Option<DbConnection>,
    pub connected: bool,
    pub identity: Option<Identity>,
    pub own_character_id: Option<u64>,
    pub tx: Sender<NetEvent>,
    pub rx: Mutex<Receiver<NetEvent>>,
    pub last_sent_pos: (f32, f32),
    pub last_sent_at: Instant,
    pub last_attack_at: Instant,
}

impl NetState {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let now = Instant::now();
        Self {
            conn: None,
            connected: false,
            identity: None,
            own_character_id: None,
            tx,
            rx: Mutex::new(rx),
            last_sent_pos: (0.0, 0.0),
            last_sent_at: now,
            last_attack_at: now,
        }
    }

    pub fn sender(&self) -> Sender<NetEvent> {
        self.tx.clone()
    }
}
