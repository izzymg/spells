pub mod events;
pub mod render;
pub mod controls;
pub mod replication;
pub mod debug;
pub mod dev_scenes;
pub mod editor;
pub mod game;
pub mod input;
pub mod ui;
pub mod window;
pub mod world_connection;

use bevy::{log::LogPlugin, prelude::*};
use std::{env, error::Error};


#[derive(SystemSet, Clone, Copy, Hash, Eq, PartialEq, Debug)]
enum SystemSets {
    NetFetch,
    NetSend,
    Controls,
    Input,
    Replication,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let mut app = App::new();

    app.configure_sets(
        Update,
        (
            SystemSets::NetFetch,
            SystemSets::Replication,
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
        ui::UiPlugin,
        input::InputPlugin,
        window::WindowPlugin,
        events::EventsPlugin,
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
            "replication" => {
                app.add_plugins(dev_scenes::DevScenesPlugin {
                    scene: dev_scenes::Scene::Replication,
                });
            },
            "gameui" => {
                app.add_plugins(dev_scenes::DevScenesPlugin {
                    scene: dev_scenes::Scene::GameUI
                });
            }
            _ => {
                panic!("unrecognised: {}", mode)
            }
        }
    } else {
        app.add_plugins(game::GamePlugin);
    }
    app.run();
    Ok(())
}
