mod render;
use crate::{events, game::GameStates, window};
use bevy::prelude::*;

fn sys_setup(mut ns: ResMut<NextState<window::WindowContext>>) {
    ns.set(window::WindowContext::Play);
}

fn sys_exit_multiplayer(mut ns: ResMut<NextState<GameStates>>) {
    ns.set(GameStates::MainMenu);
}

pub struct MultiplayerPlugin;

impl Plugin for MultiplayerPlugin {
    fn build(&self, app: &mut App) {
        // enter game
        app.add_systems(
            OnEnter(GameStates::Game),
            (
                sys_setup,
                render::sys_create_map,
                render::sys_follow_cam_predicted_player,
            ),
        );

        // exit game
        app.add_systems(OnExit(GameStates::Game), render::sys_cleanup);

        // game update
        app.add_systems(
            Update,
            (
                render::sys_add_player_rendering,
                sys_exit_multiplayer.run_if(on_event::<events::DisconnectedEvent>()),
            )
                .run_if(in_state(GameStates::Game)),
        );
    }
}
