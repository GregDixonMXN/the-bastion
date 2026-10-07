#![allow(dead_code, unused_imports, unused_variables)]

use spacetimedb::{reducer, ReducerContext, Table};

use crate::ai::enemy_ai::{tick_enemy_ai_schedule, TickEnemyAiSchedule};
use crate::tables::spawn::spawn_point;
use crate::tables::spawn::SpawnPointTable;

/// AI ticks per respawn-second (tick runs every 2 s).
const TICKS_PER_SEC: u64 = 2;

/// Registers a fixed spawn point. Open to any caller for now (GMs, events,
/// tests) — world builders will own this once tooling exists.
#[reducer]
pub fn define_spawn_point(
    ctx: &ReducerContext,
    enemy_type: String,
    pos_x: f32,
    pos_z: f32,
    respawn_secs: u64,
) {
    ctx.db.spawn_point().insert(SpawnPointTable {
        id: 0, // auto_inc
        enemy_type,
        pos_x,
        pos_y: 0.0,
        pos_z,
        respawn_secs: respawn_secs.max(10),
        alive_enemy_id: None,
        cooldown_ticks: 0,
    });
}

/// Seeds the starter field: five Gloomrat points in a loose ring outside
/// camp (2-minute repop) plus two Thornwolf dens further out (3-minute
/// repop). Idempotent per enemy type — reruns only fill gaps. Also arms the
/// 2 s enemy-AI schedule on first run.
#[reducer]
pub fn init_world(ctx: &ReducerContext) {
    use crate::tables::spawn::spawn_point;

    seed_points(
        ctx,
        "gloomrat",
        &[(18.0, 35.0), (-20.0, 40.0), (30.0, 55.0), (-8.0, 60.0), (8.0, 75.0)],
        120,
    );
    seed_points(ctx, "thornwolf", &[(55.0, 85.0), (-52.0, 88.0)], 180);

    // Arm the AI schedule (one row — reruns are no-ops while it lives).
    if ctx.db.tick_enemy_ai_schedule().iter().count() == 0 {
        ctx.db.tick_enemy_ai_schedule().insert(TickEnemyAiSchedule {
            scheduled_id: 0,
            scheduled_at: spacetimedb::ScheduleAt::Interval(
                std::time::Duration::from_secs(2).into(),
            ),
        });
        log::info!("Enemy AI schedule armed (2 s)");
    }
}

/// Housekeeping for every spawn point, run each AI tick: validate the live
/// enemy, count down empty points, repop when the timer lapses.
pub fn tick_spawns(ctx: &ReducerContext) {
    use crate::tables::enemy::enemy;
    use crate::tables::spawn::spawn_point;

    let points: Vec<SpawnPointTable> = ctx.db.spawn_point().iter().collect();
    for mut point in points {
        let live = point.alive_enemy_id.and_then(|id| {
            ctx.db
                .enemy()
                .id()
                .find(&id)
                .filter(|e| e.enemy_type == point.enemy_type)
        });
        match live {
            Some(_) => {
                // Point held — make sure the clock is parked.
                if point.cooldown_ticks != 0 {
                    point.cooldown_ticks = 0;
                    ctx.db.spawn_point().id().update(point);
                }
            }
            None => {
                point.alive_enemy_id = None;
                if point.cooldown_ticks == 0 {
                    let id = spawn_for_point(ctx, &point);
                    point.alive_enemy_id = Some(id);
                    point.cooldown_ticks =
                        point.respawn_secs.saturating_mul(TICKS_PER_SEC);
                    ctx.db.spawn_point().id().update(point);
                } else {
                    point.cooldown_ticks -= 1;
                    ctx.db.spawn_point().id().update(point);
                }
            }
        }
    }
}

fn seed_points(
    ctx: &ReducerContext,
    enemy_type: &str,
    points: &[(f32, f32)],
    respawn_secs: u64,
) {
    use crate::tables::spawn::spawn_point;
    use crate::tables::spawn::SpawnPointTable;

    let have: usize = ctx
        .db
        .spawn_point()
        .iter()
        .filter(|p| p.enemy_type == enemy_type)
        .count();
    if have > 0 {
        return;
    }
    for (x, z) in points {
        ctx.db.spawn_point().insert(SpawnPointTable {
            id: 0,
            enemy_type: enemy_type.to_string(),
            pos_x: *x,
            pos_y: 0.0,
            pos_z: *z,
            respawn_secs,
            alive_enemy_id: None,
            cooldown_ticks: 0,
        });
    }
    log::info!("Seeded {} {enemy_type} spawn points", points.len());
}

fn spawn_for_point(ctx: &ReducerContext, point: &SpawnPointTable) -> u64 {
    use crate::tables::enemy::enemy;
    use crate::tables::enemy::EnemyTable;

    let (health, speed, damage, xp, gold) =
        crate::reducers::enemy_reducers::stats(&point.enemy_type);
    let row = ctx.db.enemy().insert(EnemyTable {
        id: 0,
        enemy_type: point.enemy_type.clone(),
        health,
        max_health: health,
        pos_x: point.pos_x,
        pos_y: 0.0,
        pos_z: point.pos_z,
        target_character: None,
        home_x: point.pos_x,
        home_z: point.pos_z,
        speed,
        damage,
        xp_value: xp,
        gold_value: gold,
    });
    row.id
}
