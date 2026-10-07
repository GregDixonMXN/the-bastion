#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::components::{Character, LocalCharacter, LocalPlayer};

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
                Text::new("Lv 1 · HP --% · Gold 0"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                HudText,
            ));
        });
}

/// Refreshes HUD each frame from component data.
fn update_hud(
    character_query: Query<&Character, With<LocalCharacter>>,
    mut hud_query: Query<&mut Text, With<HudText>>,
) {
    let Ok(mut hud_text) = hud_query.single_mut() else { return };

    if let Ok(character) = character_query.single() {
        let hp_pct = (character.health / character.max_health * 100.0) as u32;
        let dead_tag = if character.is_dead { "  [DEAD — respawn at camp]" } else { "" };
        hud_text.0 = format!(
            "Lv {} · HP {}%{}{}",
            character.level, hp_pct, dead_tag, ""
        );
    }
}
