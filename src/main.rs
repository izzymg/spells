mod game;

// use std::time::Duration;

use bevy::{log::LogPlugin, prelude::*};
use game::{health, spells::{self, StartCastingEvent}, status_effect};

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


    start_casting_write.send(StartCastingEvent::new(guy, target, 0));
    start_casting_write.send(StartCastingEvent::new(target, target, 1));
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
            status_effect::StatusEffectPlugin,
        ))
        .insert_resource(Time::<Fixed>::from_hz(2.0))
        .add_plugins(())
        .add_systems(Startup, startup)
        .run();
}
