mod game_view;
mod render;
use crate::{events, game::GameStates, render::map};
use bevy::prelude::*;

fn sys_exit_multiplayer(mut ns: ResMut<NextState<GameStates>>) {
    ns.set(GameStates::MainMenu);
}

pub struct MultiplayerPlugin;

impl Plugin for MultiplayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameStates::Game),
            (map::sys_create_map, render::sys_follow_cam_predicted_player),
        );
        app.add_systems(
            Update,
            render::sys_add_player_rendering.run_if(in_state(GameStates::Game)),
        );

        app.add_systems(
            Update,
            (
                game_view::sys_add_casting_ui,
                game_view::sys_render_casters_ui,
            )
                .run_if(in_state(GameStates::Game)),
        );

        app.add_systems(
            Update,
            sys_exit_multiplayer.run_if(on_event::<events::DisconnectedEvent>()),
        );
    }
}
