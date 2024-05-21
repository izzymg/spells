use crate::{
    controls::{cameras, wish_dir},
    render::terrain,
    replication, world_connection,
};
use bevy::prelude::*;

mod multiplayer;
mod loading;
mod main_menu;

#[derive(States, Debug, Clone, PartialEq, Eq, Default, Hash)]
enum GameStates {
    #[default]
    MainMenu,
    Loading,
    Game,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameStates>();
        app.add_plugins((
            cameras::follow_cam::FollowCameraPlugin,
            wish_dir::WishDirPlugin,
            replication::ReplicationPlugin,
            world_connection::WorldConnectionPlugin,
            main_menu::MainMenuPlugin,
            loading::LoadingPlugin,
            multiplayer::MultiplayerPlugin,
            terrain::TerrainPlugin,
        )); 
    }
}
