#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::components::{LocalMech, LocalPlayer, Mech};

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
                Text::new("Sentence: -- hrs\nFuel: --%\nHealth: --%"),
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
    mech_query: Query<&Mech, With<LocalMech>>,
    player_query: Query<&LocalPlayer>,
    mut hud_query: Query<&mut Text, With<HudText>>,
) {
    let Ok(mut hud_text) = hud_query.single_mut() else { return };

    let (fuel_pct, health_pct) = if let Ok(mech) = mech_query.single() {
        (
            (mech.fuel / mech.max_fuel * 100.0) as u32,
            (mech.health / mech.max_health * 100.0) as u32,
        )
    } else {
        (0, 0)
    };

    let (sentence, rogue_tag) = if let Ok(player) = player_query.single() {
        let hrs = player.sentence_remaining / 3600;
        let tag = if player.is_rogue { "  [ROGUE]" } else { "" };
        (hrs, tag.to_string())
    } else {
        (0, String::new())
    };

    hud_text.0 = format!(
        "Sentence: {} hrs{}\nFuel: {}%\nHealth: {}%",
        sentence, rogue_tag, fuel_pct, health_pct
    );
}
