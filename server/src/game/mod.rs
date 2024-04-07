use std::error::Error;

/// snapshots of world
use bevy::{
    app::{self, FixedUpdate, Startup},
    ecs::schedule::{IntoSystemSetConfigs, SystemSet},
    log::LogPlugin,
    time::{Fixed, Time},
    MinimalPlugins,
};

pub mod alignment;
pub mod auras;
pub mod effects;
pub mod health;
pub mod serialize;
pub mod socket;
pub mod spells;
pub mod world;

/// Defines ordering of system processing across the game server.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ServerSets {
    EntityProcessing,  // despawn dead, etc
    EffectCreation,    // creation of effect events (e.g. fireball at Bob for 32 dmg)
    EffectProcessing,  // simulation & processing of events
    EffectApplication, // application of processed events
}

fn startup() {}

pub fn run_game_server() -> Result<(), Box<dyn Error>> {
    app::App::new()
        .add_plugins((
            MinimalPlugins,
            LogPlugin {
                filter: "".into(),
                level: bevy::log::Level::DEBUG,
                update_subscriber: None,
            },
            spells::SpellsPlugin,
            health::HealthPlugin,
            auras::AuraPlugin,
            effects::EffectPlugin,
            world::WorldPlugin,
        ))
        .configure_sets(
            FixedUpdate,
            ServerSets::EntityProcessing.before(ServerSets::EffectProcessing),
        )
        .insert_resource(Time::<Fixed>::from_hz(0.5))
        .add_systems(Startup, startup)
        .run();
    Ok(())
}
