#![allow(dead_code, unused_imports)]

use bevy::prelude::*;
use crate::resources::{GameState, NetState};
use crate::systems::connection::initiate_connection;
use crate::systems::net_sync::drain_net_events;

pub struct SpacetimePlugin;

impl Plugin for SpacetimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_net_state, initiate_connection).chain())
            .add_systems(Update, drain_net_events);
    }
}

fn setup_net_state(mut commands: Commands) {
    commands.insert_resource(NetState::new());
}
