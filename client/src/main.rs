mod ui;
mod world_connection;
use std::error::Error;

use bevy::prelude::*;
use world_connection::WorldConnectionPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
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
        app.configure_sets(
            Update,
            (
                GameStates::Menu.run_if(|s: Res<GameState>| s.0 == GameStates::Menu),
                GameStates::Game.run_if(|s: Res<GameState>| s.0 == GameStates::Game),
            ),
        );
        app.run();
        Ok(())
    }
}
