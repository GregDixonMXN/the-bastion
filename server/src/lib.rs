#![allow(dead_code, unused_imports)]

pub mod ai;
pub mod reducers;
pub mod tables;

// Re-export everything so the generated client SDK picks it all up.
pub use ai::kaiju_ai::*;
pub use reducers::kaiju_reducers::*;
pub use reducers::loot_reducers::*;
pub use reducers::mech_reducers::*;
pub use reducers::player_reducers::*;
pub use reducers::wall_reducers::*;
pub use tables::KaijuTable;
pub use tables::LootTable;
pub use tables::MechTable;
pub use tables::PlayerTable;
pub use tables::WallTable;
