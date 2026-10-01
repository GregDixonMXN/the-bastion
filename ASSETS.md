# Asset Guide — The Bastion

This file tells you exactly where to drop art assets so Bevy can find them.

All assets live under `client/assets/`.

---

## Directory Layout

```
client/assets/
├── models/
│   ├── mechs/
│   │   ├── light_scout.glb       ← LightScout mech
│   │   └── heavy_juggernaut.glb  ← HeavyJuggernaut mech
│   ├── kaiju/
│   │   ├── breaker.glb           ← Breaker-class Kaiju
│   │   └── parasite.glb          ← Parasite-class Kaiju
│   ├── environment/
│   │   ├── wall_segment.glb      ← Defensive wall piece
│   │   ├── bastion_gate.glb      ← Refuel/respawn gate
│   │   └── ground_tile.glb       ← Optional ground tile
│   └── loot/
│       ├── kaiju_core.glb
│       ├── scrap_metal.glb
│       ├── fuel_cell.glb
│       └── mech_part.glb
├── textures/
│   ├── mechs/
│   │   └── light_scout_albedo.png
│   ├── kaiju/
│   │   └── breaker_albedo.png
│   └── environment/
│       ├── wall_pristine.png
│       ├── wall_damaged.png
│       └── ground.png
├── audio/
│   ├── kaiju_roar.ogg
│   ├── mech_footstep.ogg
│   └── weapon_fire.ogg
└── fonts/
    └── bastion_ui.ttf            ← UI font (any TTF/OTF)
```

---

## Loading a GLB Model in Bevy

```rust
// In a startup system:
fn spawn_mech(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(SceneRoot(
        asset_server.load("models/mechs/light_scout.glb#Scene0")
    ));
}
```

For models with multiple named scenes:
```rust
asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/mechs/light_scout.glb"))
```

---

## Swapping Placeholder Cubes for Models

The current Phase 1 code spawns coloured cubes (`Cuboid`) as placeholders.
Search for `Cuboid::new` in:

- `client/src/plugins/mech_plugin.rs` — player mech
- `client/src/plugins/kaiju_plugin.rs` — Kaiju
- `client/src/plugins/wall_plugin.rs` — wall segments

Replace the `Mesh3d(meshes.add(Cuboid::new(...)))` + `MeshMaterial3d` pair with a `SceneRoot` pointing at the corresponding `.glb` asset path above.

---

## Recommended Free Asset Sources

- [Quaternius](https://quaternius.com) — low-poly mechs, robots, monsters (CC0)
- [Kenney.nl](https://kenney.nl) — sci-fi kits (CC0)
- [Sketchfab](https://sketchfab.com) — community models (check licences)
- [itch.io game assets](https://itch.io/game-assets) — various

Export all models as **GLTF 2.0 (`.glb`)** with embedded textures for easiest Bevy integration.
