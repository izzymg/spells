use crate::{
    controls::{cameras, wish_dir},
    render::map,
    replication,
    world_connection,
};
use bevy::prelude::*;

mod game_ui;
mod loading_ui;
mod main_menu_ui;
mod render;

#[derive(States, Debug, Clone, PartialEq, Eq, Default, Hash)]
enum GameStates {
    #[default]
    MainMenu,
    Loading,
    Game,
}
#[derive(Component, Debug, Default)]
struct GameObject;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameStates>();
        app.add_plugins((
            cameras::follow_cam::FollowCameraPlugin,
            wish_dir::WishDirPlugin,
            main_menu_ui::MainMenuPlugin,
            loading_ui::LoadingUIPlugin,
            game_ui::GameUIPlugin,
            replication::ReplicationPlugin,
            world_connection::WorldConnectionPlugin,
        ));

        // TODO: plugin-ise these
        app.add_systems(OnEnter(GameStates::Game), (map::sys_create_map, render::sys_follow_cam_predicted_player));
        app.add_systems(
            Update,
            render::sys_add_player_rendering.run_if(in_state(GameStates::Game)),
        );
    }
}
