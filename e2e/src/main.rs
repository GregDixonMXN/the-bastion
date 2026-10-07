mod generated;

use generated::{
    attack_enemy_reducer::attack_enemy, character_table::CharacterTableAccess,
    collect_loot_reducer::collect_loot, damage_character_reducer::damage_character,
    define_spawn_point_reducer::define_spawn_point, enemy_table::EnemyTableAccess,
    loot_table::LootTableAccess, move_character_reducer::move_character,
    register_player_reducer::register_player, respawn_reducer::respawn,
    spawn_character_reducer::spawn_character, spawn_point_table::SpawnPointTableAccess,
    DbConnection,
};
use spacetimedb_sdk::{DbContext, Identity, Table, TableWithPrimaryKey};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const URL: &str = "ws://127.0.0.1:3000";
const DB: &str = "bastionlands";

fn wait_for(label: &str, timeout: Duration, mut cond: impl FnMut() -> bool) {
    let start = Instant::now();
    while !cond() {
        if start.elapsed() > timeout {
            panic!("TIMEOUT waiting for {label}");
        }
        std::thread::sleep(Duration::from_millis(300));
    }
    println!("ok — {label}");
}

fn main() {
    let identity: Arc<Mutex<Option<Identity>>> = Arc::new(Mutex::new(None));
    let identity_cb = identity.clone();
    let conn = DbConnection::builder()
        .with_uri(URL)
        .with_module_name(DB)
        .on_connect(move |conn, id, _token| {
            println!("connected as {id}");
            *identity_cb.lock().unwrap() = Some(id);
            conn.reducers.register_player("Playtester".to_string()).unwrap();
            conn.subscription_builder()
                .on_applied(|ctx| {
                    ctx.reducers.spawn_character("Testblade".to_string()).unwrap();
                })
                .on_error(|_ctx, e| panic!("subscription failed: {e}"))
                .subscribe([
                    "SELECT * FROM player",
                    "SELECT * FROM character",
                    "SELECT * FROM enemy",
                    "SELECT * FROM loot",
                    "SELECT * FROM spawn_point",
                ]);
        })
        .on_connect_error(|_ctx, e| panic!("connect failed: {e}"))
        .on_disconnect(|_ctx, e| panic!("disconnected: {e:?}"))
        .build()
        .expect("build connection");
    conn.run_threaded();

    // --- spawn ---
    wait_for("character row", Duration::from_secs(20), || {
        let id = identity.lock().unwrap().clone();
        id.is_some_and(|id| conn.db.character().iter().any(|c| c.owner_identity == id))
    });
    let me = {
        let id = identity.lock().unwrap().clone().unwrap();
        conn.db.character().iter().find(|c| c.owner_identity == id).unwrap()
    };
    println!("character {} at ({}, {})", me.id, me.pos_x, me.pos_z);
    assert_eq!(me.level, 1);

    // --- teleport rejection (server authority) ---
    conn.reducers.move_character(me.id, 500.0, 500.0).unwrap();
    std::thread::sleep(Duration::from_secs(1));
    let me2 = conn.db.character().id().find(&me.id).unwrap();
    assert!(
        me2.pos_x.abs() < 1.0 && me2.pos_z.abs() < 1.0,
        "500m teleport must be rejected, at ({}, {})",
        me2.pos_x,
        me2.pos_z
    );
    println!("ok — 500m teleport rejected");

    // --- test spawn point near camp, 10 s repop ---
    conn.reducers.define_spawn_point("gloomrat".to_string(), 5.0, 5.0, 10).unwrap();
    wait_for("rat pops", Duration::from_secs(15), || {
        conn.db.enemy().iter().any(|e| e.home_x == 5.0 && e.home_z == 5.0)
    });
    let rat = conn
        .db
        .enemy()
        .iter()
        .find(|e| e.home_x == 5.0 && e.home_z == 5.0)
        .unwrap();
    println!("rat {} at ({}, {})", rat.id, rat.pos_x, rat.pos_z);

    // --- walk over (legal steps), let it close, and kill it ---
    conn.reducers.move_character(me.id, 5.0, 5.0).unwrap();
    wait_for("rat closes to melee", Duration::from_secs(30), || {
        let (Some(m), Some(r)) = (
            conn.db.character().id().find(&me.id),
            conn.db.enemy().id().find(&rat.id),
        ) else {
            return false;
        };
        let dx = r.pos_x - m.pos_x;
        let dz = r.pos_z - m.pos_z;
        (dx * dx + dz * dz).sqrt() <= 3.0
    });
    // 25 dmg/swing vs 60 hp: three landed hits.
    for _ in 0..8 {
        conn.reducers.attack_enemy(me.id, rat.id).unwrap();
        std::thread::sleep(Duration::from_secs(1));
        if conn.db.enemy().id().find(&rat.id).is_none() {
            break;
        }
    }
    assert!(conn.db.enemy().id().find(&rat.id).is_none(), "rat must die");
    println!("ok — rat killed");
    let me3 = conn.db.character().id().find(&me.id).unwrap();
    assert_eq!(me3.xp, 25, "kill grants 25 xp");
    println!("ok — xp granted");
    wait_for("gold loot drops", Duration::from_secs(10), || {
        conn.db.loot().iter().any(|l| !l.is_collected)
    });
    let loot = conn.db.loot().iter().find(|l| !l.is_collected).unwrap();
    conn.reducers.collect_loot(loot.id, me.id).unwrap();
    std::thread::sleep(Duration::from_secs(1));
    let me4 = conn.db.character().id().find(&me.id).unwrap();
    assert_eq!(me4.gold, loot.value, "gold credited");
    println!("ok — loot collected ({}g)", loot.value);

    // --- leash: provoke the repop, drag it past 45 m, watch it give up ---
    wait_for("repop", Duration::from_secs(30), || {
        conn.db
            .enemy()
            .iter()
            .any(|e| e.id != rat.id && e.home_x == 5.0 && e.home_z == 5.0)
    });
    let rat2 = conn
        .db
        .enemy()
        .iter()
        .find(|e| e.id != rat.id && e.home_x == 5.0 && e.home_z == 5.0)
        .unwrap();
    println!("repop rat {} — leash test", rat2.id);
    conn.reducers.attack_enemy(me.id, rat2.id).unwrap();
    std::thread::sleep(Duration::from_secs(1));
    // Drag away in legal steps: (5,5) -> (75,75) is ~99 m from home.
    for (x, z) in [
        (15.0, 15.0),
        (25.0, 25.0),
        (35.0, 35.0),
        (45.0, 45.0),
        (55.0, 55.0),
        (65.0, 65.0),
        (75.0, 75.0),
    ] {
        conn.reducers.move_character(me.id, x, z).unwrap();
        std::thread::sleep(Duration::from_millis(400));
    }
    wait_for("leash breaks", Duration::from_secs(30), || {
        conn.db.enemy().id().find(&rat2.id).is_none_or(|e| e.target_character.is_none())
    });
    println!("ok — leash broke, rat walks home");

    // --- death & respawn ---
    conn.reducers.damage_character(me.id, 9999.0).unwrap();
    std::thread::sleep(Duration::from_secs(1));
    let dead = conn.db.character().id().find(&me.id).unwrap();
    assert!(dead.is_dead, "character must be dead");
    println!("ok — death recorded (gold dropped)");
    conn.reducers.respawn(me.id).unwrap();
    std::thread::sleep(Duration::from_secs(1));
    let back = conn.db.character().id().find(&me.id).unwrap();
    assert!(!back.is_dead && back.health == back.max_health, "respawn restores");
    assert!(back.pos_x.abs() < 1.0 && back.pos_z.abs() < 1.0, "respawn at camp");
    println!("ok — respawned at camp with full health");

    println!("PLAYTEST PASSED: hunt, loot, leash, death, respawn, repop");
}
