#![allow(dead_code, unused_imports, unused_variables)]

use spacetimedb::{reducer, spacetimedb_lib::Identity, ReducerContext, Table};

use crate::tables::kaiju::kaiju;
use crate::tables::kaiju::KaijuTable;
use crate::tables::wall::wall;

/// Spawns a Kaiju at the given position targeting a specific wall segment.
#[reducer]
pub fn spawn_kaiju(
    ctx: &ReducerContext,
    kaiju_type: String,
    pos_x: f32,
    pos_z: f32,
    target_wall: String,
) {
    let (health, speed, damage) = match kaiju_type.as_str() {
        "Parasite" => (500.0, 4.0, 25.0),
        _ => (2000.0, 1.5, 80.0), // "Breaker" default
    };
    ctx.db.kaiju().insert(KaijuTable {
        id: 0, // auto_inc
        kaiju_type,
        health,
        max_health: health,
        pos_x,
        pos_y: 0.0,
        pos_z,
        target_wall_segment: target_wall,
        aggro_list: "[]".to_string(),
        speed,
        damage,
    });
}

/// Moves a Kaiju one step toward its target wall segment.
/// If adjacent to the wall, deals damage to it.
#[reducer]
pub fn move_kaiju_toward_wall(ctx: &ReducerContext, kaiju_id: u64) {
    use crate::tables::kaiju::kaiju;
    use crate::tables::wall::wall;

    let mut k = match ctx.db.kaiju().id().find(&kaiju_id) {
        Some(k) => k,
        None => return,
    };
    let w = match ctx.db.wall().segment_id().find(&k.target_wall_segment) {
        Some(w) => w,
        None => return,
    };

    let target_x = w.grid_x as f32 * 10.0;
    let target_z = w.grid_z as f32 * 10.0;

    let dx = target_x - k.pos_x;
    let dz = target_z - k.pos_z;
    let dist = (dx * dx + dz * dz).sqrt();

    if dist <= k.speed {
        crate::reducers::wall_reducers::damage_wall(
            ctx,
            k.target_wall_segment.clone(),
            k.damage,
        );
    } else {
        k.pos_x += (dx / dist) * k.speed;
        k.pos_z += (dz / dist) * k.speed;
        ctx.db.kaiju().id().update(k);
    }
}

/// Deals damage to a Kaiju and records the attacker in the aggro list.
#[reducer]
pub fn damage_kaiju(ctx: &ReducerContext, kaiju_id: u64, amount: f32, attacker: Identity) {
    use crate::tables::kaiju::kaiju;

    if let Some(mut k) = ctx.db.kaiju().id().find(&kaiju_id) {
        k.health = (k.health - amount).max(0.0);

        let attacker_str = format!("{:?}", attacker);
        let mut aggro: Vec<String> =
            serde_json::from_str(&k.aggro_list).unwrap_or_default();
        if !aggro.contains(&attacker_str) {
            aggro.push(attacker_str);
        }
        k.aggro_list = serde_json::to_string(&aggro).unwrap_or_else(|_| "[]".to_string());

        if k.health <= 0.0 {
            let (pos_x, pos_z) = (k.pos_x, k.pos_z);
            ctx.db.kaiju().id().delete(&kaiju_id);
            crate::reducers::loot_reducers::spawn_kaiju_core(ctx, pos_x, pos_z, kaiju_id);
            log::info!("Kaiju {} killed", kaiju_id);
        } else {
            ctx.db.kaiju().id().update(k);
        }
    }
}

/// Removes a dead Kaiju, spawns loot, and awards the top-aggro player.
#[reducer]
pub fn kill_kaiju(ctx: &ReducerContext, kaiju_id: u64) {
    use crate::tables::kaiju::kaiju;

    let (drop_x, drop_z) = if let Some(k) = ctx.db.kaiju().id().find(&kaiju_id) {
        let pos = (k.pos_x, k.pos_z);
        ctx.db.kaiju().id().delete(&kaiju_id);
        pos
    } else {
        (0.0, 0.0)
    };

    crate::reducers::loot_reducers::spawn_kaiju_core(ctx, drop_x, drop_z, kaiju_id);
    log::info!("Kaiju {} killed via kill_kaiju, loot spawned", kaiju_id);
}
