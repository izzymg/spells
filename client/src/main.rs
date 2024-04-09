mod world_connection;
mod ui;
use std::error::Error;

use bevy::prelude::*;
use world_connection::WorldConnectionPlugin;

#[derive(Debug, PartialEq, Eq)]
pub enum GameStates {
    Menu,
    Game,
}

#[derive(Resource)]
pub struct GameState(pub GameStates);


fn main() -> Result<(), Box<dyn Error>> {
    {
        let mut app = bevy::app::App::new();
        app.add_plugins((DefaultPlugins, WorldConnectionPlugin, ui::UiPlugin));
        app.insert_resource(GameState(GameStates::Menu));

        app.run();
        Ok(())
    }
}
