#![allow(dead_code, unused_imports, unused_variables)]

use spacetimedb::{reducer, spacetimedb_lib::Identity, ReducerContext, Table};

use crate::tables::loot::loot;
use crate::tables::loot::LootTable;

/// Spawns loot on the ground (kills, events, GM gifts).
#[reducer]
pub fn spawn_loot(
    ctx: &ReducerContext,
    loot_type: String,
    pos_x: f32,
    pos_z: f32,
    value: u64,
    from_enemy: Option<u64>,
) {
    ctx.db.loot().insert(LootTable {
        id: 0, // auto_inc
        loot_type,
        pos_x,
        pos_y: 0.0,
        pos_z,
        value,
        dropped_by_enemy_id: from_enemy,
        is_collected: false,
    });
}

/// Picks up loot within reach. Gold goes to the collector's character —
/// identity comes from the connection, and reach is server-checked.
pub const PICKUP_RANGE: f32 = 2.5;

#[reducer]
pub fn collect_loot(ctx: &ReducerContext, loot_id: u64, character_id: u64) {
    use crate::tables::character::character;
    use crate::tables::loot::loot;

    let mut character = match ctx.db.character().id().find(&character_id) {
        Some(c) if c.owner_identity == ctx.sender && !c.is_dead => c,
        _ => return,
    };
    if let Some(mut item) = ctx.db.loot().id().find(&loot_id) {
        if item.is_collected {
            return;
        }
        let dx = item.pos_x - character.pos_x;
        let dz = item.pos_z - character.pos_z;
        if (dx * dx + dz * dz).sqrt() > PICKUP_RANGE {
            return;
        }
        item.is_collected = true;
        character.gold += item.value;
        ctx.db.loot().id().update(item);
        ctx.db.character().id().update(character);
    }
}
