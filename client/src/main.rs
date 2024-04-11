pub mod game;
pub mod ui;
pub mod world_connection;
use std::error::Error;

use bevy::{log::LogPlugin, prelude::*};
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
        app.add_plugins((WorldConnectionPlugin, ui::UiPlugin, game::GamePlugin));

        #[cfg(debug_assertions)]
        app.add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::DEBUG,
            filter: "debug,wgpu_core=warn,wgpu_hal=warn,spells=info".into(),
            ..Default::default()
        }));

        #[cfg(not(debug_assertions))]
        app.add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::INFO,
            filter: "info,wgpu_core=warn,wgpu_hal=warn".into(),
            ..Default::default()
        }));

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
