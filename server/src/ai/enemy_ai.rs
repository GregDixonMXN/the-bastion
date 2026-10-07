#![allow(dead_code, unused_imports, unused_variables)]

use spacetimedb::{reducer, ReducerContext, Table};

use crate::tables::character::character;
use crate::tables::enemy::enemy;

/// How far an enemy notices a living character.
pub const AGGRO_RADIUS: f32 = 30.0;

/// AI tick — each enemy either closes on its target or, if idle, notices the
/// nearest living character in range and takes an interest.
///
/// Phase 2: schedule this via a SpacetimeDB schedule table to run every 2s:
/// ```rust
/// #[table(name = tick_enemy_ai_schedule, scheduled(tick_enemy_ai))]
/// pub struct TickEnemyAiSchedule {
///     #[primary_key] #[auto_inc] pub scheduled_id: u64,
///     pub scheduled_at: spacetimedb::ScheduleAt,
/// }
/// ```
/// Then in the `init` reducer insert a row with `ScheduleAt::Interval(Duration::from_secs(2))`.
#[reducer]
pub fn tick_enemy_ai(ctx: &ReducerContext) {
    use crate::tables::character::character;
    use crate::tables::enemy::enemy;

    // Idle enemies notice the nearest living character in radius.
    let characters: Vec<_> = ctx
        .db
        .character()
        .iter()
        .filter(|c| !c.is_dead)
        .map(|c| (c.id, c.pos_x, c.pos_z))
        .collect();
    let idle: Vec<u64> = ctx
        .db
        .enemy()
        .iter()
        .filter(|e| e.target_character.is_none())
        .map(|e| e.id)
        .collect();
    for enemy_id in idle {
        if let Some(mut e) = ctx.db.enemy().id().find(&enemy_id) {
            let mut best: Option<(u64, f32)> = None;
            for (cid, cx, cz) in &characters {
                let dx = cx - e.pos_x;
                let dz = cz - e.pos_z;
                let dist = (dx * dx + dz * dz).sqrt();
                if dist <= AGGRO_RADIUS && best.is_none_or(|(_, d)| dist < d) {
                    best = Some((*cid, dist));
                }
            }
            if let Some((cid, _)) = best {
                e.target_character = Some(cid);
                ctx.db.enemy().id().update(e);
            }
        }
    }

    let enemy_ids: Vec<u64> = ctx.db.enemy().iter().map(|e| e.id).collect();
    for id in enemy_ids {
        crate::reducers::enemy_reducers::step_enemy(ctx, id);
    }
}
