#![allow(dead_code, unused_imports, unused_variables)]

use spacetimedb::{reducer, ReducerContext, Table};

use crate::tables::kaiju::kaiju;
use crate::tables::kaiju::KaijuTable;

/// AI tick — moves every living Kaiju one step toward its target wall.
///
/// Phase 2: schedule this via a SpacetimeDB schedule table to run every 2s:
/// ```rust
/// #[table(name = tick_kaiju_ai_schedule, scheduled(tick_kaiju_ai))]
/// pub struct TickKaijuAiSchedule {
///     #[primary_key] #[auto_inc] pub scheduled_id: u64,
///     pub scheduled_at: spacetimedb::ScheduleAt,
/// }
/// ```
/// Then in the `init` reducer insert a row with `ScheduleAt::Interval(Duration::from_secs(2))`.
#[reducer]
pub fn tick_kaiju_ai(ctx: &ReducerContext) {
    use crate::tables::kaiju::kaiju;
    let kaiju_ids: Vec<u64> = ctx.db.kaiju().iter().map(|k| k.id).collect();
    for id in kaiju_ids {
        crate::reducers::kaiju_reducers::move_kaiju_toward_wall(ctx, id);
    }
}
