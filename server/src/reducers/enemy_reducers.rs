#![allow(dead_code, unused_imports, unused_variables)]

use spacetimedb::{reducer, ReducerContext, Table};

use crate::tables::enemy::enemy;
use crate::tables::enemy::EnemyTable;

/// First enemy of the game. Gloomrats infest the fields outside the starter
/// camp — weak alone, brave in packs (packs arrive with the wave director).
fn stats(enemy_type: &str) -> (f32, f32, f32, u64, u64) {
    // health, speed, damage, xp, gold
    match enemy_type {
        "gloomrat" => (60.0, 3.0, 8.0, 25, 5),
        _ => (60.0, 3.0, 8.0, 25, 5), // unknown keys spawn a gloomrat
    }
}

/// Spawns an enemy. Open to any caller for now (GMs, events, tests) — the
/// wave director will own this once it exists.
#[reducer]
pub fn spawn_enemy(ctx: &ReducerContext, enemy_type: String, pos_x: f32, pos_z: f32) {
    let (health, speed, damage, xp, gold) = stats(&enemy_type);
    ctx.db.enemy().insert(EnemyTable {
        id: 0, // auto_inc
        enemy_type,
        health,
        max_health: health,
        pos_x,
        pos_y: 0.0,
        pos_z,
        target_character: None,
        speed,
        damage,
        xp_value: xp,
        gold_value: gold,
    });
}

/// Steps one enemy toward its target (or holds position). Called by the AI
/// tick; adjacent enemies strike instead of moving.
pub fn step_enemy(ctx: &ReducerContext, enemy_id: u64) {
    use crate::tables::character::character;
    use crate::tables::enemy::enemy;

    let mut e = match ctx.db.enemy().id().find(&enemy_id) {
        Some(e) => e,
        None => return,
    };
    // Drop targets that died or vanished.
    if let Some(target) = e.target_character {
        let valid = ctx
            .db
            .character()
            .id()
            .find(&target)
            .is_some_and(|c| !c.is_dead);
        if !valid {
            e.target_character = None;
        }
    }
    let target = match e.target_character {
        Some(t) => t,
        None => return, // no quarry — holds ground until provoked or directed
    };
    let c = match ctx.db.character().id().find(&target) {
        Some(c) => c,
        None => return,
    };
    let dx = c.pos_x - e.pos_x;
    let dz = c.pos_z - e.pos_z;
    let dist = (dx * dx + dz * dz).sqrt();
    if dist <= 2.0 {
        crate::reducers::character_reducers::damage_character(ctx, target, e.damage);
    } else {
        e.pos_x += (dx / dist) * e.speed;
        e.pos_z += (dz / dist) * e.speed;
        ctx.db.enemy().id().update(e);
    }
}
