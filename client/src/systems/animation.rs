#![allow(dead_code, unused_imports)]

use bevy::animation::graph::{AnimationGraph, AnimationGraphHandle, AnimationNodeIndex};
use bevy::animation::{AnimationPlayer, RepeatAnimation};
use bevy::prelude::*;
use std::collections::HashMap;

use crate::components::{Character, Enemy};
use super::locomotion::Locomotion;

/// Clip slots on the adventurer rig. Fox drives walk/survey only.
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
    fn name(&self) -> &'static str {
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

/// Preloaded GLB handles + built animation graph, shared by all spawns.
#[derive(Resource)]
pub struct ClipLibrary {
    adventurer_gltf: Handle<Gltf>,
    fox_gltf: Handle<Gltf>,
    graph: Option<BuiltClips>,
}

pub struct BuiltClips {
    pub graph: Handle<AnimationGraph>,
    pub adventurer: HashMap<Clip, AnimationNodeIndex>,
    pub fox_walk: AnimationNodeIndex,
    pub fox_survey: AnimationNodeIndex,
}

/// Marker: needs AnimationPlayer + clip set from the library.
#[derive(Component, Debug, Clone, Default)]
pub struct NeedsClips;

/// One-shot attack flash so the swing clip plays through once.
#[derive(Component, Debug, Clone)]
pub struct AttackFlash {
    pub t: f32,
}

/// Currently playing clip, to avoid restarting every frame.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Playing(pub Clip);

/// Fox idle/walk state.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoxPose {
    Survey,
    Walk,
}

pub fn load_clip_library(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ClipLibrary {
        adventurer_gltf: asset_server.load("models/characters/adventurer.glb"),
        fox_gltf: asset_server.load("models/enemies/gloomrat.glb"),
        graph: None,
    });
}

fn build_library(
    mut library: ResMut<ClipLibrary>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    if library.graph.is_some() {
        return;
    }
    let (Some(adventurer), Some(fox)) = (
        gltfs.get(&library.adventurer_gltf),
        gltfs.get(&library.fox_gltf),
    ) else {
        return; // still loading
    };
    let mut graph = AnimationGraph::new();
    let mut adventurer_map = HashMap::new();
    for clip in [
        Clip::Idle,
        Clip::Walk,
        Clip::Run,
        Clip::Attack,
        Clip::Hurt,
        Clip::Pickup,
        Clip::Emote,
    ] {
        match adventurer.named_animations.get(clip.name()) {
            Some(handle) => {
                let idx = graph.add_clip(handle.clone(), 1.0, graph.root);
                adventurer_map.insert(clip, idx);
            }
            None => log::warn!("adventurer GLB missing clip `{}`", clip.name()),
        }
    }
    let Some(walk_handle) = fox
        .named_animations
        .get("Walk")
        .or_else(|| fox.named_animations.get("walk"))
        .cloned()
    else {
        log::warn!("fox GLB missing Walk clip");
        return;
    };
    let Some(survey_handle) = fox
        .named_animations
        .get("Survey")
        .or_else(|| fox.named_animations.get("survey"))
        .cloned()
    else {
        log::warn!("fox GLB missing Survey clip");
        return;
    };
    let fox_walk = graph.add_clip(walk_handle, 1.0, graph.root);
    let fox_survey = graph.add_clip(survey_handle, 1.0, graph.root);
    library.graph = Some(BuiltClips {
        graph: graphs.add(graph),
        adventurer: adventurer_map,
        fox_walk,
        fox_survey,
    });
    log::info!("clip library built");
}

fn play_loop(player: &mut AnimationPlayer, node: AnimationNodeIndex) {
    // NOTE: AnimationTransitions is private in bevy_animation 0.16, so clip
    // changes pop instead of blending. Revisit on the Bevy upgrade.
    player.play(node).set_repeat(RepeatAnimation::Forever);
}

fn play_once(player: &mut AnimationPlayer, node: AnimationNodeIndex) {
    player.play(node).set_repeat(RepeatAnimation::Never);
}

/// Setup: give freshly spawned models their player. Idle starts on first
/// drive tick.
fn setup_clips(
    mut commands: Commands,
    library: Res<ClipLibrary>,
    characters: Query<Entity, (With<Character>, With<NeedsClips>, Without<AnimationPlayer>)>,
    enemies: Query<Entity, (With<Enemy>, With<NeedsClips>, Without<AnimationPlayer>)>,
) {
    let Some(built) = library.graph.as_ref() else { return };
    for entity in &characters {
        commands.entity(entity).insert((
            AnimationPlayer::default(),
            
            AnimationGraphHandle(built.graph.clone()),
            Playing(Clip::Idle),
        ));
        commands.entity(entity).remove::<NeedsClips>();
    }
    for entity in &enemies {
        commands.entity(entity).insert((
            AnimationPlayer::default(),
            
            AnimationGraphHandle(built.graph.clone()),
            FoxPose::Survey,
        ));
        commands.entity(entity).remove::<NeedsClips>();
    }
}

/// Drive adventurer clips from game state: attacks play once, locomotion
/// picks run/walk/idle. Death holds its pose (tip-over arrives separately).
fn drive_characters(
    library: Res<ClipLibrary>,
    mut query: Query<(
        &Character,
        &Locomotion,
        &mut AnimationPlayer,
        &mut Playing,
        Option<&AttackFlash>,
    )>,
) {
    let Some(built) = library.graph.as_ref() else { return };
    for (character, loco, mut player, mut playing, flash) in &mut query {
        if character.is_dead {
            continue;
        }
        let want = if flash.is_some() {
            Clip::Attack
        } else {
            let speed = loco.vel.length();
            if speed > 4.0 {
                Clip::Run
            } else if speed > 0.5 {
                Clip::Walk
            } else {
                Clip::Idle
            }
        };
        if want != playing.0 {
            if let Some(node) = built.adventurer.get(&want) {
                if want == Clip::Attack {
                    play_once(&mut player, *node);
                } else {
                    play_loop(&mut player, *node);
                }
                playing.0 = want;
            }
        }
    }
}

/// Attack flashes expire back into locomotion-driven clips.
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

/// Foxes survey at rest and walk when something moves them.
fn drive_foxes(
    library: Res<ClipLibrary>,
    mut query: Query<(
        &Locomotion,
        &mut AnimationPlayer,
        &mut FoxPose,
    )>,
) {
    let Some(built) = library.graph.as_ref() else { return };
    for (loco, mut player, mut pose) in &mut query {
        let want = if loco.vel.length() > 0.5 {
            FoxPose::Walk
        } else {
            FoxPose::Survey
        };
        if want != *pose {
            let node = match want {
                FoxPose::Walk => built.fox_walk,
                FoxPose::Survey => built.fox_survey,
            };
            play_loop(&mut player, node);
            *pose = want;
        }
    }
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_clip_library).add_systems(
            Update,
            (
                build_library,
                setup_clips,
                drive_characters,
                drive_foxes,
                attack_decay,
            ),
        );
    }
}
