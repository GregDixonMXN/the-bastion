#![allow(dead_code, unused_imports, unused_variables)]

use spacetimedb::{reducer, table, ReducerContext, Table};

use crate::tables::character::character;
use crate::tables::enemy::enemy;

/// Schedule row driving the enemy AI loop every 2 seconds. Armed once by
/// `init_world`; the row living means the loop lives.
#[table(name = tick_enemy_ai_schedule, scheduled(tick_enemy_ai))]
pub struct TickEnemyAiSchedule {
    #[primary_key]
    #[auto_inc]
    pub scheduled_id: u64,
    pub scheduled_at: spacetimedb::ScheduleAt,
}

/// How far an enemy notices a living character.
pub const AGGRO_RADIUS: f32 = 30.0;

/// AI tick — repop due spawn points, then step every living enemy.
#[reducer]
pub fn tick_enemy_ai(ctx: &ReducerContext, _schedule: TickEnemyAiSchedule) {
    use crate::tables::character::character;
    use crate::tables::enemy::enemy;

    crate::reducers::spawn_reducers::tick_spawns(ctx);

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
