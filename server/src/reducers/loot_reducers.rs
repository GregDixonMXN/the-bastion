#![allow(dead_code, unused_imports, unused_variables)]

use spacetimedb::{reducer, spacetimedb_lib::Identity, ReducerContext, Table};

use crate::tables::loot::loot;
use crate::tables::loot::LootTable;

/// Spawns a loot item at the given world position.
#[reducer]
pub fn spawn_loot(
    ctx: &ReducerContext,
    loot_type: String,
    pos_x: f32,
    pos_z: f32,
    value: u64,
    from_kaiju: Option<u64>,
) {
    ctx.db.loot().insert(LootTable {
        id: 0, // auto_inc
        loot_type,
        pos_x,
        pos_y: 0.0,
        pos_z,
        value,
        dropped_by_kaiju_id: from_kaiju,
        is_collected: false,
    });
}

/// Internal helper — spawns a KaijuCore drop (not a client-callable reducer).
pub fn spawn_kaiju_core(ctx: &ReducerContext, pos_x: f32, pos_z: f32, kaiju_id: u64) {
    use crate::tables::loot::loot;
    ctx.db.loot().insert(LootTable {
        id: 0,
        loot_type: "KaijuCore".to_string(),
        pos_x,
        pos_y: 0.0,
        pos_z,
        value: 500,
        dropped_by_kaiju_id: Some(kaiju_id),
        is_collected: false,
    });
}

/// Marks a loot item as collected and credits the collector.
#[reducer]
pub fn collect_loot(ctx: &ReducerContext, loot_id: u64, collector: Identity) {
    use crate::tables::loot::loot;
    if let Some(mut item) = ctx.db.loot().id().find(&loot_id) {
        if item.is_collected {
            return;
        }
        item.is_collected = true;
        let value = item.value;
        ctx.db.loot().id().update(item);
        crate::reducers::player_reducers::update_sentence(ctx, collector, value);
        log::info!("Loot {} collected by {:?}", loot_id, collector);
    }
}
