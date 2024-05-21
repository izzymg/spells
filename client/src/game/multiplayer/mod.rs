mod game_view;
mod render;
use crate::{events, game::GameStates};
use bevy::prelude::*;

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
                game_view::sys_add_casting_ui,
                game_view::sys_render_casters_ui,
                render::sys_add_player_rendering,
                sys_exit_multiplayer.run_if(on_event::<events::DisconnectedEvent>()),
            )
                .run_if(in_state(GameStates::Game)),
        );
    }
}
