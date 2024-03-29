mod game;

use std::time::Duration;

use bevy::prelude::*;
use game::health::HealthTickSingle;

use crate::game::health::Health;

fn startup(mut commands: Commands, spell_list: Res<game::resources::SpellList>) {

    let spellcaster = game::spellcasting::Spellcaster {
        spellbook: [0, 1].to_vec(),
    };

    let casting_spell = spell_list.get_spell(spellcaster.get_spellbook_spell(0));

    println!(
        "creating spellcaster to cast {} for {:?}",
        casting_spell.name, casting_spell.cast_time
    );

    let target_entity = commands.spawn(Health(5));
    let casting = game::spellcasting::Casting {
        cast_timer: Timer::new(casting_spell.cast_time, TimerMode::Once),
        spellbook_index: 0,
        target: target_entity.id(),
    };

    commands.spawn((spellcaster, casting));
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(game::resources::get_spell_list_resource())
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(500)))
        .add_systems(Startup, startup)
        .add_systems(
            FixedUpdate,
            (
                game::health::death_system,
                game::health::health_tick_system.before(game::health::death_system),
                game::spellcasting::spell_cast_system,
                game::cleanup::cleanup_system::<HealthTickSingle>
            ),
        )
        .run();
}
