use std::error::Error;

/// snapshots of world
use bevy::{app, log::LogPlugin, prelude::*};

use self::scenes::sys_many_effects;

pub mod assets;
pub mod components;
pub mod effect_application;
pub mod effect_creation;
pub mod effect_processing;
pub mod entity_processing;
pub mod events;
pub mod net;
pub mod scenes;

/// Defines ordering of system processing across the game server.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ServerSets {
    EntityProcessing,  // despawn dead, etc
    EffectCreation,    // creation of effect events (e.g. fireball at Bob for 32 dmg)
    EffectProcessing,  // simulation & processing of events
    EffectApplication, // application of processed events
}

pub fn run_game_server() -> Result<(), Box<dyn Error>> {
    app::App::new()
        .add_plugins((
            MinimalPlugins,
            LogPlugin {
                filter: "".into(),
                level: bevy::log::Level::DEBUG,
                update_subscriber: None,
            },
            events::GameEventsPlugin,
            net::NetPlugin,
            effect_processing::EffectPlugin,
            effect_creation::EffectCreationPlugin,
            effect_application::EffectApplicationPlugin,
            entity_processing::EntityProcessingPlugin,
            assets::AssetsPlugin,
        ))
        .configure_sets(
            FixedUpdate,
            ServerSets::EntityProcessing
                .before(ServerSets::EffectCreation)
                .before(ServerSets::EffectProcessing)
                .before(ServerSets::EffectApplication),
        )
        .insert_resource(Time::<Fixed>::from_hz(0.5))
        .add_systems(Startup, sys_many_effects)
        .run();
    Ok(())
}
