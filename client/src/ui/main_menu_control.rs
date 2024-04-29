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

pub(super) fn sys_update_connection_status(
    _world_conn: Res<world_connection::WorldConnection>,
    _ui_status: ResMut<ConnectionStatus>,
    _next_game_state: ResMut<NextState<GameStates>>,
) {
}
