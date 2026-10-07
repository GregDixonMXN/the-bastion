mod generated;

use generated::{
    character_table::CharacterTableAccess, move_character_reducer::move_character,
    register_player_reducer::register_player, spawn_character_reducer::spawn_character,
    DbConnection,
};
use spacetimedb_sdk::{DbContext, Table, TableWithPrimaryKey};
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

struct Peer {
    conn: DbConnection,
    name: String,
    character_id: Option<u64>,
}

fn connect(player: &str, avatar: &str) -> Peer {
    let conn = DbConnection::builder()
        .with_uri(URL)
        .with_module_name(DB)
        .on_connect({
            let player = player.to_string();
            let avatar = avatar.to_string();
            move |conn, identity, _token| {
                println!("{player} connected as {identity}");
                conn.reducers.register_player(player.clone()).unwrap();
                conn.subscription_builder()
                    .on_applied({
                        let avatar = avatar.clone();
                        move |ctx| {
                            ctx.reducers.spawn_character(avatar.clone()).unwrap();
                        }
                    })
                    .on_error(|_ctx, e| panic!("subscription failed: {e}"))
                    .subscribe([
                        "SELECT * FROM player",
                        "SELECT * FROM character",
                        "SELECT * FROM enemy",
                        "SELECT * FROM loot",
                        "SELECT * FROM spawn_point",
                    ]);
            }
        })
        .on_connect_error(|_ctx, e| panic!("connect failed: {e}"))
        .on_disconnect(|_ctx, e| panic!("disconnected: {e:?}"))
        .build()
        .expect("build connection");
    conn.run_threaded();
    Peer {
        conn,
        name: avatar.to_string(),
        character_id: None,
    }
}

fn main() {
    let mut alice = connect("Alice", "AliceBlade");
    wait_for("alice spawns", Duration::from_secs(20), || {
        if let Some(row) = alice.conn.db.character().iter().find(|c| c.name == alice.name) {
            alice.character_id = Some(row.id);
            true
        } else {
            false
        }
    });
    let alice_id = alice.character_id.unwrap();

    // Bob joins and must see Alice already standing in the world.
    let mut bob = connect("Bob", "BobBlade");
    wait_for("bob sees alice", Duration::from_secs(20), || {
        if let Some(row) = bob.conn.db.character().iter().find(|c| c.name == alice.name) {
            bob.character_id = bob
                .conn
                .db
                .character()
                .iter()
                .find(|c| c.name == bob.name)
                .map(|c| c.id);
            let _ = row;
            true
        } else {
            false
        }
    });
    wait_for("bob spawns", Duration::from_secs(20), || {
        if let Some(row) = bob.conn.db.character().iter().find(|c| c.name == bob.name) {
            bob.character_id = Some(row.id);
            true
        } else {
            false
        }
    });

    // Alice walks 10 m; Bob must observe the new position (cross-client sync).
    alice.conn.reducers.move_character(alice_id, 10.0, 0.0).unwrap();
    wait_for("bob sees alice move", Duration::from_secs(15), || {
        bob.conn
            .db
            .character()
            .id()
            .find(&alice_id)
            .is_some_and(|c| (c.pos_x - 10.0).abs() < 0.01)
    });

    // And back the other way: Bob moves, Alice observes.
    let bob_id = bob.character_id.unwrap();
    bob.conn.reducers.move_character(bob_id, -10.0, 0.0).unwrap();
    wait_for("alice sees bob move", Duration::from_secs(15), || {
        alice
            .conn
            .db
            .character()
            .id()
            .find(&bob_id)
            .is_some_and(|c| (c.pos_x + 10.0).abs() < 0.01)
    });

    println!("ONLINE TEST PASSED: two clients share one world");
}
