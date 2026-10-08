#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::components::{Character, LocalCharacter};
use crate::resources::NetState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud)
            .add_systems(Update, update_hud);
    }
}

#[derive(Component)]
struct HudText;

/// Spawns the HUD text node.
fn spawn_hud(mut commands: Commands) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("Starting…"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                HudText,
            ));
        });
}

/// Refreshes HUD each frame. Shows connection progress honestly so a stuck
/// state names itself instead of freezing on one word.
fn update_hud(
    time: Res<Time>,
    net: Res<NetState>,
    character_query: Query<&Character, With<LocalCharacter>>,
    all_characters: Query<&Character, Without<LocalCharacter>>,
    mut hud_query: Query<&mut Text, With<HudText>>,
    mut census_t: Local<f32>,
) {
    // 10 s census: the one line that explains any "frozen" report.
    *census_t += time.delta_secs();
    if *census_t > 10.0 {
        *census_t = 0.0;
        log::info!(
            "census: local_tags={} remote_chars={} connected={} own={:?}",
            character_query.iter().count(),
            all_characters.iter().count(),
            net.connected,
            net.own_character_id,
        );
    }
    let Ok(mut hud_text) = hud_query.single_mut() else { return };

    if !net.connected {
        hud_text.0 = "Connecting to SpacetimeDB (ws://127.0.0.1:3000)…\nIs `spacetime start` running?".to_string();
        return;
    }
    let Ok(character) = character_query.single() else {
        hud_text.0 = "Connected — syncing world…".to_string();
        return;
    };
    let hp_pct = (character.health / character.max_health * 100.0) as u32;
    let xp_need = character.level as u64 * 100;
    let dead_tag = if character.is_dead { "  [DEAD — press R]" } else { "" };
    hud_text.0 = format!(
        "{} · Lv {} · XP {}/{} · HP {}% · {}g{}",
        character.name, character.level, character.xp, xp_need, hp_pct, character.gold, dead_tag
    );
}
