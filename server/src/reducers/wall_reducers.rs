#![allow(dead_code, unused_imports, unused_variables)]

use spacetimedb::{reducer, spacetimedb_lib::Identity, ReducerContext, Table};

use crate::tables::wall::wall;
use crate::tables::wall::WallTable;

const WALL_MAX_HEALTH: f32 = 1000.0;
const ROW_LABELS: &[char] = &['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J'];

/// Seeds the WallTable with a 10×10 grid (A-0 … J-9).
#[reducer]
pub fn init_walls(ctx: &ReducerContext) {
    for (row_idx, row_char) in ROW_LABELS.iter().enumerate() {
        for col in 0..10_i32 {
            let segment_id = format!("{}-{}", row_char, col);
            if ctx.db.wall().segment_id().find(&segment_id).is_some() {
                continue;
            }
            ctx.db.wall().insert(WallTable {
                segment_id,
                health: WALL_MAX_HEALTH,
                max_health: WALL_MAX_HEALTH,
                integrity: 1.0,
                visual_damage_state: 0,
                grid_x: col,
                grid_z: row_idx as i32,
            });
        }
    }
    log::info!("Wall grid initialised (10×10)");
}

/// Deals damage to a wall segment and updates derived fields.
#[reducer]
pub fn damage_wall(ctx: &ReducerContext, segment_id: String, amount: f32) {
    use crate::tables::wall::wall;
    if let Some(mut w) = ctx.db.wall().segment_id().find(&segment_id) {
        w.health = (w.health - amount).max(0.0);
        w.integrity = w.health / w.max_health;
        w.visual_damage_state = match w.integrity {
            i if i > 0.66 => 0,
            i if i > 0.33 => 1,
            i if i > 0.0  => 2,
            _              => 3,
        };
        ctx.db.wall().segment_id().update(w);
    }
}

/// Repairs a wall segment and awards sentence reduction to the repairer.
#[reducer]
pub fn repair_wall(ctx: &ReducerContext, segment_id: String, amount: f32, repairer: Identity) {
    use crate::tables::wall::wall;
    if let Some(mut w) = ctx.db.wall().segment_id().find(&segment_id) {
        w.health = (w.health + amount).min(w.max_health);
        w.integrity = w.health / w.max_health;
        w.visual_damage_state = match w.integrity {
            i if i > 0.66 => 0,
            i if i > 0.33 => 1,
            i if i > 0.0  => 2,
            _              => 3,
        };
        ctx.db.wall().segment_id().update(w);

        let reward = (amount / 10.0) as u64;
        crate::reducers::player_reducers::update_sentence(ctx, repairer, reward);
    }
}
