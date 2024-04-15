pub mod game;
pub mod render;
pub mod ui;
pub mod window;
pub mod world_connection;
pub mod editor;
pub mod controls;

use bevy::{log::LogPlugin, prelude::*};
use std::{env, error::Error};

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

    // Diagnostics
    #[cfg(debug_assertions)]
    {
        app.add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::DEBUG,
            filter: "info,wgpu_core=warn,wgpu_hal=warn,spells=debug".into(),
            ..Default::default()
        }));
        app.add_plugins((
            iyes_perf_ui::PerfUiPlugin,
            bevy::diagnostic::FrameTimeDiagnosticsPlugin,
            bevy::diagnostic::EntityCountDiagnosticsPlugin,
            bevy::diagnostic::SystemInformationDiagnosticsPlugin,
        ));

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(iyes_perf_ui::PerfUiCompleteBundle::default());
        });
    }

    #[cfg(not(debug_assertions))]
    app.add_plugins(DefaultPlugins.set(LogPlugin {
        level: bevy::log::Level::INFO,
        filter: "info,wgpu_core=warn,wgpu_hal=warn".into(),
        ..Default::default()
    }));

    app.insert_resource(GameState(GameStates::Menu));
    app.add_plugins((render::RenderPlugin, window::WindowPlugin, ui::UiPlugin));
    app.configure_sets(
        Update,
        (
            GameStates::Menu.run_if(|s: Res<GameState>| s.0 == GameStates::Menu),
            GameStates::Game.run_if(|s: Res<GameState>| s.0 == GameStates::Game),
        ),
    );

    if let Some(mode) = args.get(1) {
        match mode.as_str() {
            "editor" => {
                app.add_plugins(editor::EditorPlugin);
            }
            _ => {
                println!("unrecognised: {}", mode)
            }
        }
    } else {
        app.add_plugins((
            world_connection::WorldConnectionPlugin,
            game::GamePlugin,
        ));
    }
    app.run();
    Ok(())
}
