use bevy::prelude::*;
use crate::{world_connection, GameStates};

#[derive(Resource, Debug, Default)]
pub struct ConnectionStatus {
    pub status: String,
}

/// View wants to call connect
#[derive(Debug, Event)]
pub struct ConnectEvent {
    pub address: String,
}

pub(super) fn sys_menu_connect_ev(
    mut commands: Commands,
    mut ev_r: EventReader<ConnectEvent>,
    world_conn: Res<world_connection::WorldConnection>,
    mut status: ResMut<ConnectionStatus>,
) {
    if let Some(ev) = ev_r.read().last() {
        commands.run_system_with_input(world_conn.connect_system, (ev.address.clone(), None));
        status.status = "connecting...".into();
    }
}

pub(super) fn sys_handle_connected(
    mut connected_ev_r: EventReader<world_connection::ConnectedEvent>,
    mut ui_status: ResMut<ConnectionStatus>,
    mut next_game_state: ResMut<NextState<GameStates>>,
) {
    if connected_ev_r.read().next().is_some() {
        ui_status.status = "connected".into();
        next_game_state.set(GameStates::Game);
    }
}

pub(super) fn sys_handle_disconnected(
    mut connected_ev_r: EventReader<world_connection::DisconnectedEvent>,
    mut ui_status: ResMut<ConnectionStatus>,
) {
    if let Some(dc) = connected_ev_r.read().next() {
        if let Some(err) = &dc.0 {
            ui_status.status = format!("disconnected: {}", err);
        } else {
            ui_status.status = "disconnected for unknown reason".into();
        }
    }
}
