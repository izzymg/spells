mod game;
mod tests;

// use std::time::Duration;

use bevy::{log::LogPlugin, prelude::*};
use game::{status_effect, health, spells};

// create entities
fn startup(
    mut commands: Commands,
    mut ev_w_spellcasting: EventWriter<spells::StartCastingEvent>,
    mut ev_w_aura_add: EventWriter<status_effect::AddStatusEffectEvent>,
) {
    let target = commands.spawn(health::Health(1000)).id();

    let caster = commands
        .spawn((health::Health(50), spells::Spellcaster {}))
        .id();

    ev_w_spellcasting.send(spells::StartCastingEvent {
        entity: caster,
        target,
        spell_id: 1,
    });

    ev_w_aura_add.send(status_effect::AddStatusEffectEvent {
        target_entity: target,
        status_id: 0,
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
