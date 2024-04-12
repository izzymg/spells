pub mod game;
pub mod render;
pub mod ui;
pub mod world_connection;
use std::{env, error::Error};

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
    let args: Vec<String> = env::args().collect();
    let mut app = App::new();

    #[cfg(debug_assertions)]
    app.add_plugins(DefaultPlugins.set(LogPlugin {
        level: bevy::log::Level::DEBUG,
        filter: "info,wgpu_core=warn,wgpu_hal=warn,spells=debug".into(),
        ..Default::default()
    }));

    #[cfg(not(debug_assertions))]
    app.add_plugins(DefaultPlugins.set(LogPlugin {
        level: bevy::log::Level::INFO,
        filter: "info,wgpu_core=warn,wgpu_hal=warn".into(),
        ..Default::default()
    }));

    if let Some(mode) = args.get(1) {
        match mode.as_str() {
            "render" => {}
            _ => {
                println!("unrecognised: {}", mode)
            }
        }
    } else {
        app.add_plugins((WorldConnectionPlugin, ui::UiPlugin, game::GamePlugin));
        app.insert_resource(GameState(GameStates::Menu));
        app.configure_sets(
            Update,
            (
                GameStates::Menu.run_if(|s: Res<GameState>| s.0 == GameStates::Menu),
                GameStates::Game.run_if(|s: Res<GameState>| s.0 == GameStates::Game),
            ),
        );
        app.run();
    }
    Ok(())
}
