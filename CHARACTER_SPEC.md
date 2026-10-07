# Character & Enemy Spec — community submissions

This is the contract for community-made characters (and later, enemies).
Model it, rig it, animate it to this spec, submit it, get it approved, and it
ships in the game. Anything off-spec is returned with notes, not silently fixed.

## Philosophy

One skeleton standard, one animation list, one scale. Creativity lives in the
mesh, textures, and personality — not in the rig. If every submission shares
bones and proportions, any animation plays on any character, PvP stays fair,
and the game can grow for years without rework.

## DCC setup

- Unit: meters. Character height **1.7–1.9 m** standing (enemies: see brief).
- Model faces **+Z**, up is **+Y** (Bevy/GLTF convention).
- All transforms frozen; single root node; no N-gons on deforming meshes.

## Skeleton (required)

Humanoid biped, T-pose, 55 bones. Names are case-sensitive:

```
Root → Hips → Spine → Chest → Neck → Head
  Hips → Thigh.L/R → Calf.L/R → Foot.L/R → Toes.L/R
  Chest → Shoulder.L/R → UpperArm.L/R → Forearm.L/R → Hand.L/R
    Hand → Thumb_01..03, Index_01..03, Middle_01..03, Ring_01..03, Pinky_01..03 (per hand)
```

- Sides: `.L` / `.R` suffixes exactly as above.
- Every deforming vertex weighted to ≤ 4 bones, weights normalized.
- No scale on any bone (translation + rotation only).

## Mesh & textures

- Budget: ≤ 25k triangles for characters, ≤ 15k for starter enemies.
- One material set where possible (albedo + normal + roughness/metalness packed).
- Textures ≤ 2048px, embedded in the `.glb`.
- Export **GLB (glTF 2.0)**, embedded textures, `+Z` forward.

## Animation set (required)

Each clip separate, 30 fps, loopable where marked. Root motion baked on the
Root bone only (in-place variants also accepted and flagged):

| Clip | Length | Loop | Notes |
|---|---|---|---|
| `idle` | 2–4 s | yes | breathing weight, subtle |
| `walk` | 1–2 s cycle | yes | ~1.6 m/s feel |
| `run` | 1 s cycle | yes | ~4 m/s feel |
| `attack_melee` | 0.6–0.9 s | no | hit lands ~60% through |
| `hurt` | 0.4–0.6 s | no | flinch, returns to idle pose |
| `death` | 1.2–1.8 s | no | ends flat, stays down |
| `gather` / `pickup` | 0.8–1.2 s | no | kneel-and-take, for loot |
| `emote_wave` | 1–2 s | no | personality slot |

Clip names are the API: the game calls `attack_melee` when you swing. Wrong
names = returned.

## Submission & approval

1. Export per this spec, test the checklist below, open a PR (or submission
   thread) with: `.glb`, wireframe + textured turntable renders, and a video
   of every clip playing on YOUR mesh.
2. Reviewers run the automated gate (bone names, tri count, clip names,
   loop integrity) plus a human pass (readability at 20 m, no clipping on
   attack/hurt, personality).
3. Approved submissions merge with credit in `CREDITS.md`. Balance-affecting
   stats (speed, hitbox) are set by maintainers, never by the file.

## Self-check before submitting

- [ ] 55 bones, exact names, T-pose, +Z forward, meters
- [ ] ≤ tri budget, ≤ 4 influences/vertex, embedded 2k textures
- [ ] All 8 clips present, named exactly, 30 fps
- [ ] Attack hit lands ~60% through the clip
- [ ] Death ends flat and holds; loops loop seamlessly
- [ ] No bone scale, frozen transforms, single root

## Pipeline note

Our sister project **AssetDrop** (powertimegames.com tooling) generates
rigged, animated GLBs. Spec-compliant output from that pipeline is welcome
here through the same approval gate — no shortcuts, same checklist.
