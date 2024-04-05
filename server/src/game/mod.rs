use std::{error::Error, thread};

/// snapshots of world
use bevy::{
    app::{self, Startup}, ecs::{event::EventWriter, system::Commands}, log::LogPlugin, time::{Fixed, Time}, MinimalPlugins
};

pub mod spells;
pub mod health;
pub mod auras;
pub mod effect_application;
pub mod alignment;
pub mod world;
pub mod socket;

fn startup(
    mut commands: Commands,
    mut start_casting_write: EventWriter<spells::StartCastingEvent>,
) {
    let guy = commands
        .spawn((alignment::FactionMember(0b001), health::Health::new(50)))
        .id();

    let target = commands
        .spawn((alignment::FactionMember(0b000), health::Health::new(50)))
        .id();


    start_casting_write.send(spells::StartCastingEvent::new(guy, target, 0.into()));
    start_casting_write.send(spells::StartCastingEvent::new(target, target, 1.into()));
}

pub fn run_game_server() -> Result<(), Box<dyn Error>>{

    // find client first
    let mut client_getter = socket::client_getter::ClientGetter::create()?;

    thread::spawn(move|| {
        client_getter.block_get_client();
    });
    
    app::App::new().add_plugins((
        MinimalPlugins,
        LogPlugin {
            filter: "".into(),
            level: bevy::log::Level::DEBUG,
            update_subscriber: None,
        },
        spells::SpellsPlugin,
        health::HealthPlugin,
        auras::AuraPlugin,
        effect_application::EffectQueuePlugin,
        world::WorldPlugin,
    ))
    .insert_resource(Time::<Fixed>::from_hz(2.0))
    .add_systems(Startup, startup).run();
    Ok(())
}