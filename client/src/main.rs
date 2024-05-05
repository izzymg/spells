pub mod cameras;
pub mod debug;
pub mod dev_scenes;
pub mod editor;
pub mod game;
pub mod input;
pub mod terrain;
pub mod ui;
pub mod window;
pub mod world_connection;

use bevy::{log::LogPlugin, prelude::*};
use std::{env, error::Error};

#[derive(States, Debug, Clone, PartialEq, Eq, Default, Hash)]
pub enum GameStates {
    #[default]
    MainMenu,
    LoadGame,
    Game,
}

#[derive(SystemSet, Clone, Copy, Hash, Eq, PartialEq, Debug)]
enum SystemSets {
    NetFetch,
    NetSend,
    Controls,
    Input,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let mut app = App::new();
    app.init_state::<GameStates>();

    app.configure_sets(
        Update,
        (
            SystemSets::NetFetch,
            SystemSets::Input,
            SystemSets::Controls,
            SystemSets::NetSend,
        )
            .chain(),
    );

    // Diagnostics
    #[cfg(debug_assertions)]
    {
        app.add_plugins((
            DefaultPlugins.set(LogPlugin {
                level: bevy::log::Level::DEBUG,
                filter: "info,wgpu_core=warn,wgpu_hal=warn,spells=debug".into(),
                ..Default::default()
            }),
            debug::DebugPlugin,
        ));
    }

    // Release logging
    #[cfg(not(debug_assertions))]
    app.add_plugins(DefaultPlugins.set(LogPlugin {
        level: bevy::log::Level::INFO,
        filter: "info,wgpu_core=warn,wgpu_hal=warn".into(),
        ..Default::default()
    }));

    app.add_plugins((
        input::InputPlugin,
        terrain::TerrainPlugin,
        window::WindowPlugin,
        ui::UiPlugin,
    ));

    if let Some(mode) = args.get(1) {
        match mode.as_str() {
            "editor" => {
                app.add_plugins(editor::EditorPlugin);
            }
            "followcam" => {
                app.add_plugins(dev_scenes::DevScenesPlugin {
                    scene: dev_scenes::Scene::FollowCamera,
                });
            }
            _ => {
                panic!("unrecognised: {}", mode)
            }
        }
    } else {
        app.add_plugins((world_connection::WorldConnectionPlugin, game::GamePlugin));
    }
    app.run();
    Ok(())
}
