#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::components::{Character, LocalCharacter};

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
                Text::new("Connecting…"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                HudText,
            ));
        });
}

/// Refreshes HUD each frame from the local character.
fn update_hud(
    character_query: Query<&Character, With<LocalCharacter>>,
    mut hud_query: Query<&mut Text, With<HudText>>,
) {
    let Ok(mut hud_text) = hud_query.single_mut() else { return };

    if let Ok(character) = character_query.single() {
        let hp_pct = (character.health / character.max_health * 100.0) as u32;
        let xp_need = character.level as u64 * 100;
        let dead_tag = if character.is_dead { "  [DEAD — press R]" } else { "" };
        hud_text.0 = format!(
            "{} · Lv {} · XP {}/{} · HP {}% · {}g{}",
            character.name, character.level, character.xp, xp_need, hp_pct, character.gold, dead_tag
        );
    }
}
