#![allow(dead_code, unused_imports, unused_variables)]

use spacetimedb::{reducer, ReducerContext, Table};

use crate::tables::enemy::enemy;
use crate::tables::enemy::EnemyTable;

/// First enemies of the game. Gloomrats infest the near fields; thornwolves
/// range further out — tougher, faster, richer. Shared stat block so spawn
/// points and manual spawns agree.
pub(crate) fn stats(enemy_type: &str) -> (f32, f32, f32, u64, u64) {
    // health, speed, damage, xp, gold
    match enemy_type {
        "thornwolf" => (150.0, 3.5, 14.0, 60, 12),
        _ => (60.0, 3.0, 8.0, 25, 5), // "gloomrat" default
    }
}

/// Spawns an enemy. Open to any caller for now (GMs, events, tests) — spawn
/// points own routine population; this stays for events and tests.
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
        home_x: pos_x,
        home_z: pos_z,
        speed,
        damage,
        xp_value: xp,
        gold_value: gold,
    });
}

/// How far from home an enemy chases before giving up and walking back.
pub const LEASH_RADIUS: f32 = 45.0;

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
        None => {
            // No quarry — drift home if dragged away, else hold ground.
            return_home(ctx, &mut e);
            return;
        }
    };
    let c = match ctx.db.character().id().find(&target) {
        Some(c) => c,
        None => return,
    };
    // Leash: quarry dragged too far from home breaks off and walks back.
    let hx = c.pos_x - e.home_x;
    let hz = c.pos_z - e.home_z;
    if (hx * hx + hz * hz).sqrt() > LEASH_RADIUS {
        e.target_character = None;
        ctx.db.enemy().id().update(e);
        return;
    }
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

/// Walks a targetless enemy one step home. Called when idle or leashed.
fn return_home(ctx: &ReducerContext, e: &mut EnemyTable) {
    use crate::tables::enemy::enemy;
    let dx = e.home_x - e.pos_x;
    let dz = e.home_z - e.pos_z;
    let dist = (dx * dx + dz * dz).sqrt();
    if dist <= e.speed {
        if dist > 0.01 {
            e.pos_x = e.home_x;
            e.pos_z = e.home_z;
            // Healed by the walk home — classic reset, no free hits banked.
            e.health = e.max_health;
            e.target_character = None;
            ctx.db.enemy().id().update(e.clone());
        }
    } else {
        e.pos_x += (dx / dist) * e.speed;
        e.pos_z += (dz / dist) * e.speed;
        ctx.db.enemy().id().update(e.clone());
    }
}
