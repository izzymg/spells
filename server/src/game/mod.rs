use clap::{Parser, Subcommand};
use std::error::Error;

/// snapshots of world
use bevy::{app, log::LogPlugin, prelude::*};

pub mod assets;
pub mod effect_application;
pub mod effect_creation;
pub mod effect_processing;
pub mod entity_processing;
pub mod events;
pub mod net;
pub mod scenes;

#[derive(Parser)]
struct Cli {
    // Password required to connect to this server. Don't specify for open access.
    #[arg(short, long)]
    password: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Scene { name: String },
}

/// Defines ordering of system processing across the game server.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ServerSets {
    NetworkFetch,      // process incoming network data from clients
    EntityProcessing,  // despawn dead, etc
    EffectCreation,    // creation of effect events (e.g. fireball at Bob for 32 dmg)
    EffectProcessing,  // simulation & processing of events
    EffectApplication, // application of processed events
    NetworkSend,       // output of network data to clients
}

pub fn run_game_server() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let mut app = app::App::new();

    match &cli.command { Some(Commands::Scene { name }) => {
            if let Some(scene_sys) = scenes::get_scene(name) {
                println!("starting scene {}", name);
                app.add_systems(Startup, scene_sys);
            } else {
                return Err(format!("no scene {}", name).into());
            }
        },
        None => {
            println!("starting blank");
        },
    }

    if cli.password.is_some() {
        println!("running with password");
    } else {
        println!("! running with no password");
    }

    app.add_plugins((
        MinimalPlugins,
        LogPlugin {
            filter: "".into(),
            level: bevy::log::Level::DEBUG,
            update_subscriber: None,
        },
        events::GameEventsPlugin,
        net::NetPlugin { server_password: cli.password },
        effect_processing::EffectPlugin,
        effect_creation::EffectCreationPlugin,
        effect_application::EffectApplicationPlugin,
        entity_processing::EntityProcessingPlugin,
        assets::AssetsPlugin,
    ))
    .configure_sets(
        FixedUpdate,
        (
            ServerSets::NetworkFetch,
            ServerSets::EntityProcessing,
            ServerSets::EffectCreation,
            ServerSets::EffectProcessing,
            ServerSets::EffectApplication,
            ServerSets::NetworkSend,
        )
            .chain(),
    )
    .insert_resource(Time::<Fixed>::from_hz(4.0))
    .run();
    Ok(())
}
