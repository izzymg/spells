use lib_spells::serialization;
use bevy::{app::AppExit, log, prelude::*};

use crate::world_connection;

/// Authoratative world state.
#[derive(Resource, Debug)]
pub struct ActiveWorldState(pub Option<serialization::WorldState>);

/// Process & consume new world state data.
pub(super) fn sys_consume_world_state(mut state: ResMut<ActiveWorldState>) {
    if state.is_added() {
        log::debug!("consuming world state");
    }
    state.0 = None;
}

/// Check for new world state from the world connection.
pub(super) fn sys_check_world_server_data(
    fetch: NonSend<world_connection::WorldConnectionRx>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    mut state: ResMut<ActiveWorldState>,
) {
    // TODO: main menu, deal with invalid data
    match fetch.try_recv_data() {
        Ok(res) => {
            if let Some(data) = res {
                state.0 = Some(serialization::WorldState::deserialize(&data).unwrap());
            }
        },
        Err(err) => {
            log::info!("disconnected from server: {}", err);
        }
    }
}

pub struct WorldStatePlugin;
impl Plugin for WorldStatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ActiveWorldState(None));
        app.add_systems(Update, (
            sys_check_world_server_data,
            sys_consume_world_state,
        ).chain());
    }
}