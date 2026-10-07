# Asset Guide — Bastionlands

This file tells you exactly where to drop art assets so Bevy can find them.

All assets live under `client/assets/`.

Community-made characters and enemies must additionally satisfy
[CHARACTER_SPEC.md](./CHARACTER_SPEC.md) (skeleton, proportions, clip names)
before they are accepted — file layout alone is not approval.

---

## Directory Layout

```
client/assets/
├── models/
│   ├── characters/
│   │   └── adventurer.glb        ← Cesium Man stand-in (CC-BY, see CREDITS.md)
│   ├── enemies/
│   │   └── gloomrat.glb          ← Fox stand-in (CC0/CC-BY, see CREDITS.md)
│   ├── environment/
│   │   ├── town_gate.glb         ← wanted (cube placeholder for now)
│   │   └── tent.glb              ← wanted (cube placeholder for now)
│   └── loot/
│       └── gold_cache.glb        ← wanted (cube placeholder for now)
├── textures/
│   ├── characters/
│   │   └── adventurer_albedo.png
│   ├── enemies/
│   │   └── gloomrat_albedo.png
│   └── environment/
│       ├── ground.png
│       └── tent_canvas.png
├── audio/
│   ├── sword_swing.ogg
│   ├── sword_hit.ogg
│   ├── rat_squeal.ogg
│   ├── pickup_gold.ogg
│   ├── level_up.ogg
│   └── death.ogg
└── fonts/
    └── bastion_ui.ttf            ← UI font (any TTF/OTF)
```

---

## Loading a GLB Model in Bevy

```rust
// In a startup system:
fn spawn_adventurer(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(SceneRoot(
        asset_server.load("models/characters/adventurer.glb#Scene0")
    ));
}
```

For models with multiple named scenes:
```rust
asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/characters/adventurer.glb"))
```

---

## Swapping Placeholder Cubes for Models

The current Phase 1 code spawns coloured cubes (`Cuboid`) as placeholders.
Search for `Cuboid::new` in:

- `client/src/plugins/character_plugin.rs` — player character
- `client/src/plugins/enemy_plugin.rs` — enemies
- `client/src/plugins/camp_plugin.rs` — camp dressing (gate, tents)

Replace the `Mesh3d(meshes.add(Cuboid::new(...)))` + `MeshMaterial3d` pair with a `SceneRoot` pointing at the corresponding `.glb` asset path above.

---

## Recommended Free Asset Sources

- [Quaternius](https://quaternius.com) — low-poly heroes, animals, monsters (CC0)
- [Kenney.nl](https://kenney.nl) — fantasy and nature kits (CC0)
- [Sketchfab](https://sketchfab.com) — community models (check licences)
- [itch.io game assets](https://itch.io/game-assets) — various

Export all models as **GLTF 2.0 (`.glb`)** with embedded textures for easiest Bevy integration.

---

## Pipeline note

Sister project **AssetDrop** generates rigged, animated GLBs. Its output can
land here once it passes the CHARACTER_SPEC.md gate — same checklist as
hand-made submissions.
