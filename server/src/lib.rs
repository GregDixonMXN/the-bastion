#![allow(dead_code, unused_imports)]

pub mod ai;
pub mod reducers;
pub mod tables;

// Re-export everything so the generated client SDK picks it all up.
pub use ai::enemy_ai::*;
pub use reducers::character_reducers::*;
pub use reducers::enemy_reducers::*;
pub use reducers::loot_reducers::*;
pub use reducers::player_reducers::*;
pub use reducers::spawn_reducers::*;
pub use tables::CharacterTable;
pub use tables::EnemyTable;
pub use tables::LootTable;
pub use tables::PlayerTable;
pub use tables::SpawnPointTable;
