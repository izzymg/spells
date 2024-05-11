mod game_view;
use bevy::prelude::*;
use crate::game::GameStates;

pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                game_view::sys_add_casting_ui,
                game_view::sys_render_casters_ui,
            )
                .run_if(in_state(GameStates::Game)),
        );
    }
}
