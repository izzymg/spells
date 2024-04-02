mod game;

// use std::time::Duration;

use bevy::{log::LogPlugin, prelude::*};
use game::{auras, effect_application, health, spells};

// create entities
fn startup(
    mut commands: Commands,
    mut start_casting_write: EventWriter<spells::StartCastingEvent>,
) {
    let guy = commands
        .spawn(health::Health::new(50))
        .id();

    let target = commands
        .spawn(health::Health::new(50))
        .id();


    start_casting_write.send(spells::StartCastingEvent::new(guy, target, 0.into()));
    start_casting_write.send(spells::StartCastingEvent::new(target, target, 1.into()));
}
fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            LogPlugin {
                filter: "spells=debug".into(),
                level: bevy::log::Level::DEBUG,
                update_subscriber: None,
            },
            spells::SpellsPlugin,
            health::HealthPlugin,
            auras::AuraPlugin,
            effect_application::EffectQueuePlugin,
        ))
        .insert_resource(Time::<Fixed>::from_hz(2.0))
        .add_plugins(())
        .add_systems(Startup, startup)
        .run();
}
