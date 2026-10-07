#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use std::time::Duration;
use crate::components::{Character, Enemy, LocalCharacter, LootItem};
use crate::generated::{
    attack_enemy_reducer::attack_enemy, collect_loot_reducer::collect_loot,
    respawn_reducer::respawn, train_focus_reducer::train_focus,
    train_vitality_reducer::train_vitality, visit_healer_reducer::visit_healer,
};
use crate::resources::NetState;

const MELEE_RANGE: f32 = 3.0;
const PICKUP_RANGE: f32 = 2.5;
const SWING_COOLDOWN: Duration = Duration::from_millis(600);

/// LMB: swing at the nearest enemy in reach. E: pick up nearby loot.
/// R: respawn at camp while dead. H: camp healer (10g full mend).
/// T: train vitality (+25 max HP). Y: train focus (+10 max mana).
/// Training costs scale per rank and happen at camp — server decides all.
pub fn combat_input(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    net: Res<NetState>,
    local: Query<(&Transform, &Character), With<LocalCharacter>>,
    enemies: Query<(&Transform, &Enemy), Without<LocalCharacter>>,
    loot: Query<(&Transform, &LootItem)>,
) {
    let Some(conn) = net.conn.as_ref() else { return };
    let Some(char_id) = net.own_character_id else { return };
    let Ok((transform, character)) = local.single() else { return };
    if character.is_dead && !keyboard.just_pressed(KeyCode::KeyR) {
        return;
    }
    let here = transform.translation;

    if keyboard.just_pressed(KeyCode::KeyR) && character.is_dead {
        if let Err(e) = conn.reducers.respawn(char_id) {
            log::warn!("respawn send failed: {e}");
        }
        return;
    }
    if character.is_dead {
        return;
    }

    if mouse.just_pressed(MouseButton::Left) && net.last_attack_at.elapsed() >= SWING_COOLDOWN {
        let mut best: Option<(f32, u64)> = None;
        for (t, e) in &enemies {
            let d = t.translation.distance(here);
            if d <= MELEE_RANGE && best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, e.db_id));
            }
        }
        if let Some((_, enemy_id)) = best {
            if let Err(e) = conn.reducers.attack_enemy(char_id, enemy_id) {
                log::warn!("attack send failed: {e}");
            }
        }
    }

    if keyboard.just_pressed(KeyCode::KeyE) {
        let mut best: Option<(f32, u64)> = None;
        for (t, l) in &loot {
            let d = t.translation.distance(here);
            if d <= PICKUP_RANGE && best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, l.db_id));
            }
        }
        if let Some((_, loot_id)) = best {
            if let Err(e) = conn.reducers.collect_loot(loot_id, char_id) {
                log::warn!("collect send failed: {e}");
            }
        }
    }

    if keyboard.just_pressed(KeyCode::KeyH) {
        if character.gold >= 10 {
            if let Err(e) = conn.reducers.visit_healer(char_id) {
                log::warn!("healer send failed: {e}");
            }
        }
    }

    if keyboard.just_pressed(KeyCode::KeyT) {
        if let Err(e) = conn.reducers.train_vitality(char_id) {
            log::warn!("train send failed: {e}");
        }
    }

    if keyboard.just_pressed(KeyCode::KeyY) {
        if let Err(e) = conn.reducers.train_focus(char_id) {
            log::warn!("train send failed: {e}");
        }
    }
}
