mod game;

use std::time::Duration;

use bevy::{log::LogPlugin, prelude::*};
use game::{health, spellcasting};

// create entities
fn startup(mut commands: Commands, mut ev_w: EventWriter<spellcasting::StartCastingEvent>) {
    let target = commands.spawn(health::Health(50)).id();

    let caster = commands.spawn((
        health::Health(50),
        spellcasting::Spellcaster{},
    )).id();

    ev_w.send(spellcasting::StartCastingEvent {
        entity: caster,
        target,
        spell_id: 1,
    });

}
fn main() {
    App::new()
        .add_plugins((MinimalPlugins, LogPlugin {
            filter: "spells=debug".into(),
            level: bevy::log::Level::DEBUG, 
            update_subscriber: None,
        }))
        .insert_resource(game::resources::get_spell_list_resource())
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(500)))
        .add_event::<spellcasting::StartCastingEvent>()
        .add_event::<health::HealthTickEvent>()
        .add_systems(Startup, startup)
        .add_systems(
            FixedUpdate,
            (
                game::health::death_system,
                game::health::health_tick_system.before(game::health::death_system),
                game::spellcasting::spell_cast_system,
                game::spellcasting::start_casting_system,
            ),
        )
        .run();
}
