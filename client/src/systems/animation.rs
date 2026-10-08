#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use bevy::transform::TransformSystem;
use serde::Deserialize;
use std::collections::HashMap;

use crate::components::{Character, Enemy, LocalCharacter};
use super::locomotion::Locomotion;

/// Overgrowth-style animation, owned end to end: a Blender-baked pose bank
/// sampled by hand, root motion driving the body, analytic two-bone leg IK
/// planting the feet. Bevy's AnimationPlayer couldn't see our bones; this
/// doesn't need it to.
///
/// Bank: client/assets/anims/animbank.json — per-bone LOCAL deltas from rest
/// (translation) plus delta quats, 30 fps, root XZ normalized to cycle
/// start. Deltas compose onto Bevy rest pose through the yup axis change,
/// so no convention can silently drift them.

const BANK_JSON: &str = include_str!("../../assets/anims/animbank.json");

/// Yup axis change, Blender→game: (x, y, z) → (x, z, −y). Rotation −90°
// about X, as a quaternion conjugator for delta rotations.
fn yup_conj() -> Quat {
    Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
}

fn yup_point(v: Vec3) -> Vec3 {
    Vec3::new(v.x, v.z, -v.y)
}

/// Clip slots. Keys match bank clip names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Clip {
    Idle,
    Walk,
    Run,
    Attack,
    Hurt,
    Pickup,
    Emote,
}

impl Clip {
    fn key(&self) -> &'static str {
        match self {
            Clip::Idle => "idle",
            Clip::Walk => "walk",
            Clip::Run => "run",
            Clip::Attack => "attack_melee",
            Clip::Hurt => "hurt",
            Clip::Pickup => "pickup",
            Clip::Emote => "emote_wave",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct BankClip {
    frames: Vec<Vec<[f32; 7]>>,
    #[serde(default)]
    root_speed: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct RawBank {
    fps: f32,
    bones: Vec<String>,
    clips: HashMap<String, BankClip>,
}

/// Shared baked bank: bone names, per-clip frames of [loc3, quat4] deltas.
#[derive(Resource)]
struct AnimBank {
    fps: f32,
    bone_index: HashMap<String, usize>,
    clips: HashMap<&'static str, BankClip>,
    hips: usize,
    /// Cycle-start hips delta per clip, so the bone never drifts.
    root0: HashMap<&'static str, (f32, f32, f32)>,
}

impl AnimBank {
    fn load() -> Self {
        let raw: RawBank = serde_json::from_str(BANK_JSON).expect("animbank parses");
        let bone_index: HashMap<String, usize> = raw
            .bones
            .iter()
            .enumerate()
            .map(|(i, n)| (n.clone(), i))
            .collect();
        let mut clips = HashMap::new();
        let mut root0 = HashMap::new();
        for clip in [
            Clip::Idle,
            Clip::Walk,
            Clip::Run,
            Clip::Attack,
            Clip::Hurt,
            Clip::Pickup,
            Clip::Emote,
        ] {
            let data = &raw.clips[clip.key()];
            let hips = bone_index["mixamorig:Hips"];
            let f0 = &data.frames[0][hips];
            root0.insert(clip.key(), (f0[0], f0[1], f0[2]));
            clips.insert(clip.key(), data.clone());
        }
        log::info!(
            "anim bank: {} bones, {} clips @ {}fps",
            raw.bones.len(),
            clips.len(),
            raw.fps
        );
        Self { fps: raw.fps, bone_index: bone_index.clone(), clips, hips: bone_index["mixamorig:Hips"], root0 }
    }

    fn duration(&self, clip: Clip) -> f32 {
        self.clips[clip.key()].frames.len() as f32 / self.fps
    }
}

/// One leg chain, resolved at rig build.
#[derive(Debug, Clone)]
struct LegChain {
    hip: Entity,
    knee: Entity,
    ankle: Entity,
    hip_parent: Entity,
    upper_len: f32,
    lower_len: f32,
}

/// Per-rig state: bone entities, Bevy rest pose, playback + blend.
#[derive(Component)]
struct Rig {
    bones: Vec<Entity>,
    rest_pos: Vec<Vec3>,
    rest_rot: Vec<Quat>,
    legs: Vec<LegChain>,
    clip: Clip,
    time: f32,
    prev_clip: Clip,
    prev_time: f32,
    blend: f32,
    /// Last bank-frame position the root motion consumed, per clip track.
    /// Deltas between consumed positions make speed framerate-independent.
    last_ft: f32,
    last_track: Clip,
}

/// Marker: scene spawned, bone map not built yet.
#[derive(Component, Debug, Clone, Default)]
pub struct NeedsRig;

/// One-shot attack flash so the swing plays through once.
#[derive(Component, Debug, Clone)]
pub struct AttackFlash {
    pub t: f32,
}

fn build_bank(mut commands: Commands) {
    commands.insert_resource(AnimBank::load());
}

fn leg_entity(map: &HashMap<String, Entity>, side: &str, part: &str) -> Option<Entity> {
    map.get(&format!("mixamorig:{side}{part}")).copied()
}

/// Traverse a fresh adventurer scene: map bank bone names to entities,
/// capture Bevy rest pose, resolve leg chains with hip parents.
fn build_rigs(
    mut commands: Commands,
    bank: Res<AnimBank>,
    roots: Query<Entity, (With<Character>, With<NeedsRig>)>,
    children: Query<&Children>,
    named: Query<(Entity, &Name, &Transform)>,
    parents: Query<&ChildOf>,
) {
    for root in &roots {
        let mut stack: Vec<(Entity, Entity)> = children
            .get(root)
            .map(|c| c.to_vec().into_iter().map(|e| (e, root)).collect())
            .unwrap_or_default();
        let mut map: HashMap<String, (Entity, Entity, Vec3, Quat)> = HashMap::new();
        while let Some((entity, parent)) = stack.pop() {
            if let Ok((_, name, tf)) = named.get(entity) {
                let key = name.to_string();
                if bank.bone_index.contains_key(&key) {
                    map.insert(key, (entity, parent, tf.translation, tf.rotation));
                }
            }
            if let Ok(kids) = children.get(entity) {
                stack.extend(kids.to_vec().into_iter().map(|e| (e, entity)));
            }
        }
        if map.len() < 60 {
            continue; // scene still streaming in — retry next frame
        }
        let n = bank.bone_index.len();
        let mut bones = vec![Entity::PLACEHOLDER; n];
        let mut rest_pos = vec![Vec3::ZERO; n];
        let mut rest_rot = vec![Quat::IDENTITY; n];
        for (name, idx) in &bank.bone_index {
            if let Some((entity, _, pos, rot)) = map.get(name) {
                bones[*idx] = *entity;
                rest_pos[*idx] = *pos;
                rest_rot[*idx] = *rot;
            }
        }
        if bones.iter().any(|e| *e == Entity::PLACEHOLDER) {
            log::warn!("rig missing bones — retrying");
            continue;
        }
        let mut legs = Vec::new();
        let entities: HashMap<String, Entity> =
            map.iter().map(|(k, v)| (k.clone(), v.0)).collect();
        for side in ["Left", "Right"] {
            let key = |part: &str| format!("mixamorig:{side}{part}");
            if let (Some(&hip), Some(&knee), Some(&ankle)) = (
                entities.get(&key("UpLeg")),
                entities.get(&key("Leg")),
                entities.get(&key("Foot")),
            ) {
                // Segment lengths from rest offsets: Mixamo chains nest
                // (knee under hip, ankle under knee), so local magnitudes
                // are the true lengths.
                let hi = bank.bone_index[&key("UpLeg")];
                let ki = bank.bone_index[&key("Leg")];
                let ai = bank.bone_index[&key("Foot")];
                let hip_parent = parents.get(hip).map(|c| c.parent()).unwrap_or(root);
                legs.push(LegChain {
                    hip,
                    knee,
                    ankle,
                    hip_parent,
                    upper_len: rest_pos[ki].length(),
                    lower_len: rest_pos[ai].length(),
                });
                let _ = hi;
            }
        }
        // Segment lengths from Bevy rest offsets (nested chains: magnitudes
        // are the true lengths).
        log::info!("rig built: {} bones, {} legs", n, legs.len());
        commands.entity(root).insert(Rig {
            bones,
            rest_pos,
            rest_rot,
            legs,
            clip: Clip::Idle,
            time: 0.0,
            prev_clip: Clip::Idle,
            prev_time: 0.0,
            blend: 1.0,
            last_ft: 0.0,
            last_track: Clip::Idle,
        });
        commands.entity(root).remove::<NeedsRig>();
    }
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build_bank)
            .add_systems(
            Update,
            (
                build_rigs,
                drive_rigs,
                apply_pose,
                root_motion,
                attack_decay,
            ),
            )
            .add_systems(
                PostUpdate,
                leg_ik.after(TransformSystem::TransformPropagate),
            );
    }
}

/// Drive clip selection from game state; advance time; crossfade 0.15 s.
fn drive_rigs(
    time: Res<Time>,
    bank: Res<AnimBank>,
    mut query: Query<(&Character, &Locomotion, &mut Rig, Option<&AttackFlash>)>,
) {
    if bank.clips.is_empty() {
        return;
    }
    let dt = time.delta_secs();
    for (character, loco, mut rig, flash) in &mut query {
        if character.is_dead {
            continue;
        }
        let want = if flash.is_some() {
            Clip::Attack
        } else {
            let speed = loco.vel.length();
            if speed > 3.0 {
                Clip::Run
            } else if speed > 0.5 {
                Clip::Walk
            } else {
                Clip::Idle
            }
        };
        if want != rig.clip {
            rig.prev_clip = rig.clip;
            rig.prev_time = rig.time;
            rig.clip = want;
            rig.time = 0.0;
            rig.blend = 0.0;
            log::info!("clip -> {:?}", want);
        }
        rig.time += dt;
        let dur = bank.duration(rig.clip);
        if rig.time >= dur {
            if rig.clip == Clip::Attack {
                rig.prev_clip = rig.clip;
                rig.prev_time = 0.0;
                rig.clip = Clip::Idle;
                rig.time = 0.0;
                rig.blend = 0.0;
            } else {
                rig.time -= dur;
            }
        }
        if rig.blend < 1.0 {
            rig.prev_time += dt;
            rig.blend = (rig.blend + dt / 0.15).min(1.0);
        }
    }
}

fn sample_frame(bank: &AnimBank, clip: Clip, time: f32, bone: usize) -> ([f32; 3], Quat) {
    let data = &bank.clips[clip.key()];
    let n = data.frames.len();
    let mut ft = (time * bank.fps) % n as f32;
    if ft < 0.0 {
        ft += n as f32;
    }
    let i0 = ft.floor() as usize % n;
    let i1 = (i0 + 1) % n;
    let f = ft - ft.floor();
    let a = &data.frames[i0][bone];
    let b = &data.frames[i1][bone];
    let loc = [
        a[0] + (b[0] - a[0]) * f,
        a[1] + (b[1] - a[1]) * f,
        a[2] + (b[2] - a[2]) * f,
    ];
    let qa = Quat::from_xyzw(a[3], a[4], a[5], a[6]);
    let qb = Quat::from_xyzw(b[3], b[4], b[5], b[6]);
    (loc, qa.slerp(qb, f))
}

/// Write the blended delta pose onto every bone, composed over Bevy rest.
/// Root XZ stays pinned to cycle start — displacement travels through root
/// motion below, never the bone.
fn apply_pose(
    bank: Res<AnimBank>,
    mut rigs: Query<&mut Rig>,
    mut bones: Query<&mut Transform>,
) {
    let cq = yup_conj();
    for mut rig in &mut rigs {
        let r0 = bank.root0[rig.clip.key()];
        for (idx, entity) in rig.bones.iter().enumerate() {
            let Ok(mut tf) = bones.get_mut(*entity) else { continue };
            let (mut loc, mut rot) = sample_frame(&bank, rig.clip, rig.time, idx);
            if rig.blend < 1.0 {
                let (ploc, prot) = sample_frame(&bank, rig.prev_clip, rig.prev_time, idx);
                let b = rig.blend;
                loc = [
                    ploc[0] + (loc[0] - ploc[0]) * b,
                    ploc[1] + (loc[1] - ploc[1]) * b,
                    ploc[2] + (loc[2] - ploc[2]) * b,
                ];
                rot = prot.slerp(rot, b);
            }
            let rest_p = rig.rest_pos[idx];
            let rest_r = rig.rest_rot[idx];
            let mut off = Vec3::new(loc[0], loc[1], loc[2]);
            if idx == bank.hips {
                off.x -= r0.0;
                off.y -= r0.1;
            }
            tf.translation = rest_p + yup_point(off);
            let dq = Quat::from_xyzw(rot.x, rot.y, rot.z, rot.w);
            tf.rotation = rest_r * (cq * dq * cq.conjugate());
        }
    }
}

/// Root motion: consume bank-frame hips displacement between the last
/// consumed position and now, rotated by facing. Consuming the exact span
/// the clock advanced keeps speed framerate-independent and loop-safe.
fn root_motion(
    bank: Res<AnimBank>,
    mut query: Query<(&mut Transform, &Locomotion, &mut Rig), With<LocalCharacter>>,
) {
    for (mut transform, loco, mut rig) in &mut query {
        let data = &bank.clips[rig.clip.key()];
        let n = data.frames.len() as f32;
        let now_ft = positive_mod(rig.time * bank.fps, n);
        let (from_ft, span_ft) = if rig.last_track != rig.clip {
            // Fresh clip (or first run): anchor without teleporting.
            (now_ft, 0.0)
        } else {
            let mut span = now_ft - rig.last_ft;
            if span < 0.0 {
                span += n; // wrapped the loop seam
            }
            // Clamp runaway spans (hitches, tab-outs) to two frames.
            (rig.last_ft, span.min(2.0))
        };
        let a = sample_root_xz(&bank, rig.clip, from_ft);
        let b = sample_root_xz(&bank, rig.clip, from_ft + span_ft);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let mult = if loco.dodge_t > 0.0 { 3.0 } else { 1.0 };
        let (sy, cy) = (loco.yaw.sin(), loco.yaw.cos());
        transform.translation.x += (dx * cy + dy * sy) * mult;
        transform.translation.z += (-dx * sy + dy * cy) * mult;
        rig.last_ft = now_ft;
        rig.last_track = rig.clip;
    }
}

fn positive_mod(x: f32, n: f32) -> f32 {
    let mut r = x % n.max(1.0);
    if r < 0.0 {
        r += n.max(1.0);
    }
    r
}

/// Hips planar position at a fractional bank-frame index (bank plane).
fn sample_root_xz(bank: &AnimBank, clip: Clip, ft: f32) -> (f32, f32) {
    let data = &bank.clips[clip.key()];
    let n = data.frames.len();
    let i0 = ft.floor() as usize % n;
    let i1 = (i0 + 1) % n;
    let f = ft - ft.floor();
    let a = &data.frames[i0][bank.hips];
    let b = &data.frames[i1][bank.hips];
    (
        a[0] + (b[0] - a[0]) * f,
        a[1] + (b[1] - a[1]) * f,
    )
}

fn attack_decay(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AttackFlash)>,
) {
    for (entity, mut flash) in &mut query {
        flash.t -= time.delta_secs();
        if flash.t <= 0.0 {
            commands.entity(entity).remove::<AttackFlash>();
        }
    }
}

/// Analytic two-bone leg IK, Overgrowth post-pass style: after the pose is
/// written and globals propagated, plant stance feet on the terrain and
/// solve hip+knee rotations. Swing-phase feet (well above ground) are left
/// to the clip. Runs in PostUpdate so GlobalTransforms are fresh.
fn leg_ik(
    _bank: Res<AnimBank>,
    mut sets: ParamSet<(
        Query<(&Rig, &Transform)>,
        Query<&GlobalTransform>,
        Query<&ChildOf>,
        Query<&mut Transform>,
    )>,
) {
    let jobs: Vec<SolveJob> = sets
        .p0()
        .iter()
        .flat_map(|(rig, root_tf)| {
            let root_yaw = root_tf.rotation.to_euler(EulerRot::YXZ).0;
            rig.legs
                .iter()
                .cloned()
                .map(move |leg| SolveJob { leg, root_yaw })
        })
        .collect();
    for job in &jobs {
        solve_one_leg(job, &mut sets);
    }
}

#[derive(Clone)]
struct SolveJob {
    leg: LegChain,
    root_yaw: f32,
}

fn solve_one_leg(
    job: &SolveJob,
    sets: &mut ParamSet<(
        Query<(&Rig, &Transform)>,
        Query<&GlobalTransform>,
        Query<&ChildOf>,
        Query<&mut Transform>,
    )>,
) {
    let leg = &job.leg;
    let fwd = Vec3::new(job.root_yaw.sin(), 0.0, job.root_yaw.cos());
    let (h, a, ground, upper_dir, l1, l2) = {
        let globals = sets.p1();
        let (Ok(h_glob), Ok(a_glob)) = (globals.get(leg.hip), globals.get(leg.ankle)) else {
            return;
        };
        let (h, a) = (h_glob.translation(), a_glob.translation());
        let ground = 0.10; // flat world until the heightmap pass returns
        if a.y > ground + 0.25 {
            return; // swing phase — clip owns the foot
        }
        let target = Vec3::new(a.x, ground, a.z);
        let l1 = leg.upper_len.max(0.05);
        let l2 = leg.lower_len.max(0.05);
        let mut d = (target - h).length();
        d = d.clamp((l1 - l2).abs() + 0.01, l1 + l2 - 0.005);
        let dir = (target - h).normalize_or_zero();
        if dir == Vec3::ZERO {
            return;
        }
        // Bend plane: pole = character forward (knees point ahead).
        let mut n = dir.cross(fwd);
        if n.length_squared() < 1e-8 {
            n = dir.cross(Vec3::X);
            if n.length_squared() < 1e-8 {
                return;
            }
        }
        n = n.normalize();
        // Upper segment angle from the hip→target line.
        let cos_a = ((l1 * l1 + d * d - l2 * l2) / (2.0 * l1 * d)).clamp(-1.0, 1.0);
        let perp = n.cross(dir).normalize_or_zero();
        let mut upper_dir =
            (dir * cos_a + perp * (1.0 - cos_a * cos_a).sqrt()).normalize_or_zero();
        // Verify the knee lands ahead, else mirror the bend.
        let knee_pos = h + upper_dir * l1;
        if (knee_pos - h).dot(fwd) < 0.0 {
            upper_dir =
                (dir * cos_a - perp * (1.0 - cos_a * cos_a).sqrt()).normalize_or_zero();
        }
        (h, a, ground, upper_dir, l1, l2)
    };
    let target = Vec3::new(a.x, ground, a.z);
    solve_leg(leg, sets, h, upper_dir, target, l1);
}

/// Aim the hip segment along `upper_dir`, then hinge the knee so the ankle
/// lands on target. Rotations convert through current parent worlds.
fn solve_leg(
    leg: &LegChain,
    sets: &mut ParamSet<(
        Query<(&Rig, &Transform)>,
        Query<&GlobalTransform>,
        Query<&ChildOf>,
        Query<&mut Transform>,
    )>,
    hip_pos: Vec3,
    upper_dir: Vec3,
    target: Vec3,
    l1: f32,
) {
    let hip_parent_rot = {
        let hip_parent = {
            let parents = sets.p2();
            parents.get(leg.hip).ok().map(|c| c.parent())
        };
        let Some(hip_parent) = hip_parent else {
            return;
        };
        let globals = sets.p1();
        let Some(g) = globals.get(hip_parent).ok() else {
            return;
        };
        g.to_scale_rotation_translation().1
    };
    let knee_pos = {
        let globals = sets.p1();
        let Some(g) = globals.get(leg.knee).ok() else {
            return;
        };
        g.translation()
    };
    let cur_upper = (knee_pos - hip_pos).normalize_or_zero();
    if cur_upper == Vec3::ZERO {
        return;
    }
    let hip_local = {
        let locals = sets.p3();
        locals.get(leg.hip).map(|tf| tf.rotation).unwrap_or(Quat::IDENTITY)
    };
    let hip_world_rot = hip_parent_rot * hip_local;
    let q = Quat::from_rotation_arc(cur_upper, upper_dir);
    let hip_world_new = q * hip_world_rot;
    {
        let mut locals = sets.p3();
        if let Ok(mut tf) = locals.get_mut(leg.hip) {
            tf.rotation = hip_parent_rot.conjugate() * hip_world_new;
        }
    }
    // Knee hinge: aim lower segment at the target.
    let knee_new_pos = hip_pos + upper_dir * l1;
    let lower_want = (target - knee_new_pos).normalize_or_zero();
    if lower_want == Vec3::ZERO {
        return;
    }
    let (ankle_pos, knee_local) = {
        let ankle_pos = {
            let globals = sets.p1();
            globals.get(leg.ankle).map(|g| g.translation()).unwrap_or(target)
        };
        let knee_local = {
            let locals = sets.p3();
            locals.get(leg.knee).map(|tf| tf.rotation).unwrap_or(Quat::IDENTITY)
        };
        (ankle_pos, knee_local)
    };
    let cur_lower = (ankle_pos - knee_new_pos).normalize_or_zero();
    if cur_lower == Vec3::ZERO {
        return;
    }
    let qk = Quat::from_rotation_arc(cur_lower, lower_want);
    let knee_world_new = qk * hip_world_new * knee_local;
    {
        let mut locals = sets.p3();
        if let Ok(mut tf) = locals.get_mut(leg.knee) {
            tf.rotation = hip_world_new.conjugate() * knee_world_new;
        }
    }
}
