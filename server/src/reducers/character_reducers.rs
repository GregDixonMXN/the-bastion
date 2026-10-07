#![allow(dead_code, unused_imports, unused_variables)]

use spacetimedb::{reducer, spacetimedb_lib::Identity, ReducerContext, Table};

use crate::tables::character::character;
use crate::tables::character::CharacterTable;

/// Spawn coordinates for new and respawned characters (starter camp).
pub const SPAWN_X: f32 = 0.0;
pub const SPAWN_Z: f32 = 0.0;

/// XP needed to rise from `level` to `level + 1`.
pub fn xp_for_level(level: u32) -> u64 {
    u64::from(level) * 100
}

/// Melee damage dealt by a character at `level`.
pub fn melee_damage(level: u32) -> f32 {
    20.0 + level as f32 * 5.0
}

pub const MELEE_RANGE: f32 = 3.0;

/// Creates the sender's Adventurer. One living character per player for now —
/// calling it again while yours lives is a no-op.
#[reducer]
pub fn spawn_character(ctx: &ReducerContext, name: String) {
    use crate::tables::player::player;
    let already = ctx
        .db
        .character()
        .iter()
        .any(|c| c.owner_identity == ctx.sender && !c.is_dead);
    if already {
        return;
    }
    let clean = name.trim().chars().take(24).collect::<String>();
    let id = ctx.db.character().insert(CharacterTable {
        id: 0, // auto_inc
        owner_identity: ctx.sender,
        name: if clean.is_empty() { "Adventurer".to_string() } else { clean },
        class: "adventurer".to_string(),
        level: 1,
        xp: 0,
        gold: 0,
        health: 100.0,
        max_health: 100.0,
        mana: 50.0,
        max_mana: 50.0,
        pos_x: SPAWN_X,
        pos_y: 0.0,
        pos_z: SPAWN_Z,
        is_dead: false,
    });
    if let Some(mut p) = ctx.db.player().identity().find(&ctx.sender) {
        p.current_character_id = Some(id.id);
        ctx.db.player().identity().update(p);
    }
}

/// Moves the sender's character. You cannot move anyone else's, nor move
/// while dead. Positions are trust-but-verify: jumps over 15 m in one call
/// are rejected, so clients must send small frequent steps (the game client
/// syncs every ~1 m). Teleports stay a GM/event privilege, not a player one.
pub const MAX_STEP: f32 = 15.0;

#[reducer]
pub fn move_character(ctx: &ReducerContext, character_id: u64, new_x: f32, new_z: f32) {
    use crate::tables::character::character;
    if let Some(mut c) = ctx.db.character().id().find(&character_id) {
        if c.owner_identity != ctx.sender || c.is_dead {
            return;
        }
        let dx = new_x - c.pos_x;
        let dz = new_z - c.pos_z;
        if (dx * dx + dz * dz).sqrt() > MAX_STEP {
            log::warn!("Character {} move rejected: step too far", character_id);
            return;
        }
        c.pos_x = new_x;
        c.pos_z = new_z;
        ctx.db.character().id().update(c);
    }
}

/// Basic melee attack on an enemy. Range-checked; kills grant XP (with
/// level-ups), drop gold loot, and free the spawn point — a fresh enemy
/// repops there once its timer lapses.
#[reducer]
pub fn attack_enemy(ctx: &ReducerContext, character_id: u64, enemy_id: u64) {
    use crate::tables::character::character;
    use crate::tables::enemy::enemy;

    let character = match ctx.db.character().id().find(&character_id) {
        Some(c) if c.owner_identity == ctx.sender && !c.is_dead => c,
        _ => return,
    };
    let mut enemy = match ctx.db.enemy().id().find(&enemy_id) {
        Some(e) => e,
        None => return,
    };
    let dx = enemy.pos_x - character.pos_x;
    let dz = enemy.pos_z - character.pos_z;
    if (dx * dx + dz * dz).sqrt() > MELEE_RANGE {
        return; // out of reach — client predicts, server decides
    }
    enemy.health = (enemy.health - melee_damage(character.level)).max(0.0);
    if enemy.health <= 0.0 {
        let (xp, gold, x, z) = (enemy.xp_value, enemy.gold_value, enemy.pos_x, enemy.pos_z);
        ctx.db.enemy().id().delete(&enemy_id);
        grant_xp(ctx, character_id, xp);
        crate::reducers::loot_reducers::spawn_loot(
            ctx,
            "GoldCache".to_string(),
            x,
            z,
            gold,
            Some(enemy_id),
        );
    } else {
        // Hitting it gets its attention.
        enemy.target_character = Some(character_id);
        ctx.db.enemy().id().update(enemy);
    }
}

/// Adds XP and applies level-ups (heal to full, grow max health/mana).
pub fn grant_xp(ctx: &ReducerContext, character_id: u64, amount: u64) {
    use crate::tables::character::character;
    if let Some(mut c) = ctx.db.character().id().find(&character_id) {
        c.xp += amount;
        while c.xp >= xp_for_level(c.level) {
            c.xp -= xp_for_level(c.level);
            c.level += 1;
            c.max_health += 25.0;
            c.max_mana += 10.0;
            c.health = c.max_health;
            c.mana = c.max_mana;
            log::info!("Character {} reached level {}", character_id, c.level);
        }
        ctx.db.character().id().update(c);
    }
}

/// Damages a character (called by enemy AI and traps — never directly by
/// clients to hurt others). Death flags the body; loot/XP loss stays minimal
/// while the game is young: half gold drops where you fell.
#[reducer]
pub fn damage_character(ctx: &ReducerContext, character_id: u64, amount: f32) {
    use crate::tables::character::character;
    if let Some(mut c) = ctx.db.character().id().find(&character_id) {
        if c.is_dead {
            return;
        }
        c.health = (c.health - amount).max(0.0);
        if c.health <= 0.0 {
            c.is_dead = true;
            let dropped = c.gold / 2;
            c.gold -= dropped;
            let (x, z) = (c.pos_x, c.pos_z);
            ctx.db.character().id().update(c);
            if dropped > 0 {
                crate::reducers::loot_reducers::spawn_loot(
                    ctx,
                    "GoldCache".to_string(),
                    x,
                    z,
                    dropped,
                    None,
                );
            }
            log::info!("Character {} died", character_id);
        } else {
            ctx.db.character().id().update(c);
        }
    }
}

/// Returns the sender's dead character to the starter camp, restored.
#[reducer]
pub fn respawn(ctx: &ReducerContext, character_id: u64) {
    use crate::tables::character::character;
    if let Some(mut c) = ctx.db.character().id().find(&character_id) {
        if c.owner_identity != ctx.sender || !c.is_dead {
            return;
        }
        c.is_dead = false;
        c.health = c.max_health;
        c.mana = c.max_mana;
        c.pos_x = SPAWN_X;
        c.pos_z = SPAWN_Z;
        ctx.db.character().id().update(c);
    }
}

/// Camp healer: stand near the gate (8 m of spawn) and, for 10 gold, mend
/// to full. The first camp service — vendors and trainers follow the same
/// stand-here-pay-this shape.
pub const HEALER_RANGE: f32 = 8.0;
pub const HEALER_PRICE: u64 = 10;

#[reducer]
pub fn visit_healer(ctx: &ReducerContext, character_id: u64) {
    use crate::tables::character::character;
    if let Some(mut c) = ctx.db.character().id().find(&character_id) {
        if c.owner_identity != ctx.sender || c.is_dead {
            return;
        }
        let dx = c.pos_x - SPAWN_X;
        let dz = c.pos_z - SPAWN_Z;
        if (dx * dx + dz * dz).sqrt() > HEALER_RANGE {
            return; // the healer stays by the gate — come to them
        }
        if c.gold < HEALER_PRICE {
            return;
        }
        c.gold -= HEALER_PRICE;
        c.health = c.max_health;
        c.mana = c.max_mana;
        ctx.db.character().id().update(c);
    }
}
