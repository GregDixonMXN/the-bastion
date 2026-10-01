# The Bastion

> *High-tech island city-prison. Convict Mech Pilots defend against colossal Kaiju rising from the sea. Earn your freedom — one kill at a time.*

---

## What Is This?

**The Bastion** is a multiplayer action game where players control giant mechs defending an island prison against waves of Kaiju. Kills, repairs, and cooperation reduce your sentence (the in-game XP/currency). Attack fellow convicts and become a Rogue — hunted by automated turrets and rivals alike.

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
│       ├── tables/      # DB table definitions
│       ├── reducers/    # Server-side logic
│       └── ai/          # Kaiju AI tick
├── client/          # Bevy game client
│   └── src/
│       ├── plugins/     # Bevy plugins (one per system)
│       ├── systems/     # ECS systems
│       ├── components/  # ECS components
│       └── resources/   # Global resources
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
| **1 — Foundation** *(current)* | Project skeleton, SpacetimeDB tables & reducers, Bevy scaffold, stub visuals |
| **2 — Networking** | Wire real SpacetimeDB SDK, generated client types, live state sync |
| **3 — Gameplay** | Weapons system, mech combat, loot collection, sentence economy |
| **4 — Polish** | 3D models, sound, particle FX, UI polish, progression systems |

---

## Asset Placeholders

See **[ASSETS.md](./ASSETS.md)** for where to drop 3D models and textures.

---

## Controls (Phase 1)

| Key | Action |
|---|---|
| `W A S D` | Move mech |
| Mouse | Aim (stub) |
| `LMB` | Shoot (stub) |
