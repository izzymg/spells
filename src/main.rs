mod game;

use std::time::Duration;

use bevy::{log::LogPlugin, prelude::*};
use game::{auras, health, spells};

// create entities
fn startup(
    mut commands: Commands,
    mut ev_w_spellcasting: EventWriter<spells::StartCastingEvent>,
    mut ev_w_aura_add: EventWriter<auras::AddAuraEvent<{auras::aura_types::TICKING_HP}>>,
) {
    let target = commands.spawn(health::Health(50)).id();

    let caster = commands
        .spawn((health::Health(50), spells::Spellcaster {}))
        .id();

    // ev_w_spellcasting.send(spells::StartCastingEvent {
    //     entity: caster,
    //     target,
    //     spell_id: 1,
    // });

    ev_w_aura_add.send(auras::AddAuraEvent::<{auras::aura_types::TICKING_HP}> {
        target,
        aura_data_id: 0,
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
            auras::AurasPlugin,
            spells::SpellsPlugin,
            health::HealthPlugin,
        ))
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(1000)))
        .add_plugins(())
        .add_systems(Startup, startup)
        .run();
}
