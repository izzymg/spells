mod game;

// use std::time::Duration;

use bevy::{log::LogPlugin, prelude::*};
use game::{status_effect, health, spells};

// create entities
fn startup(
    mut commands: Commands,
    mut ev_w_aura_add: EventWriter<status_effect::AddStatusEffectEvent>,
) {
    let guy = commands
        .spawn(health::Health::new(50))
        .id();

    ev_w_aura_add.send(status_effect::AddStatusEffectEvent {
        target_entity: guy,
        status_id: 0,
    });
    ev_w_aura_add.send(status_effect::AddStatusEffectEvent {
        target_entity: guy,
        status_id: 1,
    });
    ev_w_aura_add.send(status_effect::AddStatusEffectEvent {
        target_entity: guy,
        status_id: 1,
    });
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
