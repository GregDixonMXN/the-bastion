# Bastionlands

> *A frontier town at the edge of the wilds. Take up your blade, earn your name — one hunt at a time.*

---

## What Is This?

**Bastionlands** is an open-source fantasy MMO in the spirit of OG Guild Wars
and EverQuest, built with the community, in the open. It starts deliberately
small — one starter town, one base character (the Adventurer), a field of
Gloomrats — and grows enemy by enemy, zone by zone, with player-made
characters joining the roster through the spec-and-approval pipeline in
[CHARACTER_SPEC.md](./CHARACTER_SPEC.md).

The loop, from day one: leave town, hunt, loot gold, earn XP, level up, die,
respawn, go again.

---

## Tech Stack

| Layer | Technology |
|---|---|
| Server / State | [SpacetimeDB](https://spacetimedb.com) (Rust server module) |
| Client / Visuals | [Bevy Engine](https://bevyengine.org) 0.16 (Rust) |
| Networking | SpacetimeDB SDK (auto-generated Rust client types) |

---

## Project Structure

```
the-bastion/
├── server/          # SpacetimeDB module (compiled to WASM)
│   └── src/
│       ├── tables/      # player, character, enemy, loot
│       ├── reducers/    # movement, melee, XP/levels, loot, respawn
│       └── ai/          # enemy aggro + pursuit tick
├── client/          # Bevy game client
│   └── src/
│       ├── plugins/     # Bevy plugins (one per system)
│       ├── systems/     # ECS systems
│       ├── components/  # ECS components
│       └── resources/   # Global resources
├── CHARACTER_SPEC.md    # community character/enemy submission contract
└── Makefile
```

---

## How to Run

### Prerequisites

```bash
# Install SpacetimeDB CLI
curl -sSf https://install.spacetimedb.com | sh

# Make sure Rust is installed (https://rustup.rs)
```

### 1. Start a local SpacetimeDB instance

```bash
spacetime start
```

### 2. Publish the server module

```bash
make publish
# or manually:
spacetime publish --server local the-bastion
```

### 3. Run the client

```bash
make client
# or:
cargo run -p client
```

### One-liner (publish + client)

```bash
make dev
```

---

## Headless playtest

`cargo run -p e2e` drives the full loop against the local module with no
window: register → spawn → teleport rejection → hunt → XP → loot → repop →
leash break → death → respawn. It defines its own 10-second spawn point, so
the shared field is untouched. Takes ~2 minutes.

`cargo run -p e2e --bin online` proves the multiplayer core: two connections
join, see each other's characters, and observe each other's movement.

---

## World visuals

Procedural heightmap terrain (vertex-colored meadow/rock/sand, water ring,
scattered pines and rocks — all deterministic, no downloads), starter camp
dressing, and CC-BY/CC0 stand-in models (see [CREDITS.md](./CREDITS.md)):
Cesium Man as the Adventurer, the animated Fox as the Gloomrat. Dev note:
run the client with `target/debug/assets` linked to `client/assets`
(`ln -s ../../client/assets target/debug/assets`) — Bevy resolves assets
relative to the executable.

---

## Generate Client SDK Types

After publishing the server module, generate the Rust client bindings:

```bash
spacetime generate --lang rust --out-dir client/src/generated --project-path server
```

Then update `client/src/plugins/spacetime_plugin.rs` to use the generated types (see Phase 2 comments in that file).

---

## Phase Roadmap

| Phase | Focus |
|---|---|
| **1 — World core** *(current)* | Server-authoritative character/enemy/loot tables, melee + XP/levels + respawn, enemy AI tick, Bevy scaffold |
| **2 — Networking** | Publish module, generated client types, live state sync, input→reducer calls |
| **3 — First hunt** | Starter town + Gloomrat fields playable loop, death/respawn, leash + repop tuning |
| **4 — Community roster** | First approved community character via CHARACTER_SPEC.md, second enemy, zone 2 |

---

## Asset Placeholders

See **[ASSETS.md](./ASSETS.md)** for where to drop 3D models and textures.

---

## Controls (Phase 1)

| Key | Action |
|---|---|
| `W A S D` | Move (predicted locally, synced via `move_character`) |
| `LMB` | Melee swing — nearest enemy in 3 m (`attack_enemy`, 0.6 s cooldown) |
| `E` | Pick up nearest loot in 2.5 m (`collect_loot`) |
| `R` | Respawn at camp while dead (`respawn`) |
| `H` | Camp healer — 10g full mend within 8 m of the gate (`visit_healer`) |

Thornwolves den far out (55,85 and -52,88): 150 HP, harder hits, 60 XP.
Rats near camp, wolves further — hunt within your means.

Local dev runs against `ws://127.0.0.1:3000`, module `bastionlands`.
Identities are fresh on every launch for now (no token persistence yet).
