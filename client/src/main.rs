pub mod controls;
pub mod editor;
pub mod game;
pub mod input;
pub mod render;
pub mod ui;
pub mod window;
pub mod world_connection;

use bevy::{log::LogPlugin, prelude::*};
use std::{env, error::Error};

#[derive(States, Debug, Clone, PartialEq, Eq, Default, Hash)]
pub enum GameStates {
    #[default]
    MainMenu,
    Game,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let mut app = App::new();
    app.init_state::<GameStates>();

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

    app.add_plugins((
        input::InputPlugin,
        render::RenderPlugin,
        window::WindowPlugin,
        ui::UiPlugin,
    ));

    if let Some(mode) = args.get(1) {
        match mode.as_str() {
            "editor" => {
                app.add_plugins(editor::EditorPlugin);
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
