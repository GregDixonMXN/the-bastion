#![allow(dead_code, unused_imports, unused_variables)]

use spacetimedb::{reducer, spacetimedb_lib::Identity, ReducerContext, Table};

use crate::tables::mech::mech;
use crate::tables::mech::MechTable;

const DEFAULT_MAX_HEALTH: f32 = 1000.0;
const DEFAULT_MAX_FUEL: f32 = 500.0;

/// Spawns a new mech for a player.
#[reducer]
pub fn spawn_mech(
    ctx: &ReducerContext,
    owner: Identity,
    mech_type: String,
    pos_x: f32,
    pos_z: f32,
) {
    let max_health = match mech_type.as_str() {
        "HeavyJuggernaut" => 2000.0,
        _ => DEFAULT_MAX_HEALTH,
    };
    ctx.db.mech().insert(MechTable {
        id: 0, // auto_inc
        owner_identity: owner,
        health: max_health,
        max_health,
        fuel: DEFAULT_MAX_FUEL,
        max_fuel: DEFAULT_MAX_FUEL,
        mech_type,
        pos_x,
        pos_y: 0.0,
        pos_z,
        is_static: false,
        equipped_weapons: "[]".to_string(),
    });
}

/// Moves a mech to a new position and consumes fuel proportionally.
#[reducer]
pub fn move_mech(ctx: &ReducerContext, mech_id: u64, new_x: f32, new_z: f32) {
    use crate::tables::mech::mech;
    if let Some(mut m) = ctx.db.mech().id().find(&mech_id) {
        if m.is_static {
            log::warn!("Mech {} is out of fuel and cannot move", mech_id);
            return;
        }
        let dx = new_x - m.pos_x;
        let dz = new_z - m.pos_z;
        let dist = (dx * dx + dz * dz).sqrt();
        m.pos_x = new_x;
        m.pos_z = new_z;
        ctx.db.mech().id().update(m);
        consume_fuel_inner(ctx, mech_id, dist * 0.5);
    }
}

/// Internal helper — drains fuel without being a public reducer entry point.
fn consume_fuel_inner(ctx: &ReducerContext, mech_id: u64, amount: f32) {
    use crate::tables::mech::mech;
    if let Some(mut m) = ctx.db.mech().id().find(&mech_id) {
        m.fuel = (m.fuel - amount).max(0.0);
        if m.fuel <= 0.0 {
            m.is_static = true;
            log::info!("Mech {} is now static (out of fuel)", mech_id);
        }
        ctx.db.mech().id().update(m);
    }
}

/// Drains fuel from a mech (public reducer). Sets is_static when empty.
#[reducer]
pub fn consume_fuel(ctx: &ReducerContext, mech_id: u64, amount: f32) {
    consume_fuel_inner(ctx, mech_id, amount);
}

/// Refuels a mech to its maximum fuel capacity (called at the Bastion Gate).
#[reducer]
pub fn refuel_mech(ctx: &ReducerContext, mech_id: u64) {
    use crate::tables::mech::mech;
    if let Some(mut m) = ctx.db.mech().id().find(&mech_id) {
        m.fuel = m.max_fuel;
        m.is_static = false;
        ctx.db.mech().id().update(m);
        log::info!("Mech {} refuelled", mech_id);
    }
}

/// Deals damage to a mech. Despawns it when health reaches zero.
#[reducer]
pub fn damage_mech(ctx: &ReducerContext, mech_id: u64, amount: f32) {
    use crate::tables::mech::mech;
    if let Some(mut m) = ctx.db.mech().id().find(&mech_id) {
        m.health = (m.health - amount).max(0.0);
        if m.health <= 0.0 {
            ctx.db.mech().id().delete(&mech_id);
            log::info!("Mech {} destroyed", mech_id);
        } else {
            ctx.db.mech().id().update(m);
        }
    }
}

/// PvP attack. If the target mech is destroyed the attacker becomes rogue.
#[reducer]
pub fn attack_mech(ctx: &ReducerContext, attacker_id: u64, target_id: u64) {
    use crate::tables::mech::mech;
    let attacker = match ctx.db.mech().id().find(&attacker_id) {
        Some(m) => m,
        None => return,
    };
    let target = match ctx.db.mech().id().find(&target_id) {
        Some(m) => m,
        None => return,
    };

    let dmg = 150.0_f32;
    let new_health = (target.health - dmg).max(0.0);

    if new_health <= 0.0 {
        ctx.db.mech().id().delete(&target_id);
        log::warn!(
            "Mech {} killed mech {} in PvP — attacker becomes ROGUE",
            attacker_id, target_id
        );
        crate::reducers::player_reducers::mark_rogue(ctx, attacker.owner_identity);
    } else {
        let mut updated = target;
        updated.health = new_health;
        ctx.db.mech().id().update(updated);
    }
}
