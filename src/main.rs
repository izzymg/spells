mod game;

use bevy::prelude::*;

mod health {
    use bevy::ecs::component::Component;

    #[derive(Debug, Component)]
    pub struct Damageable {
        pub hit_points: i64,
    }
}

fn startup(mut commands: Commands, spell_list: Res<game::resources::SpellList>) {

    let spellcaster = game::spellcasting::Spellcaster {
        spellbook: [0, 1].to_vec(),
    };

    let casting_spell = spell_list.get_spell(
        spellcaster.get_spellbook_spell(0),
    );

    println!("creating spellcaster to cast {} for {:?}", casting_spell.name, casting_spell.cast_time);

    let casting = game::spellcasting::Casting {
        cast_timer: Timer::new(casting_spell.cast_time, TimerMode::Once),
        spellbook_index: 0,
    };

    commands.spawn(
        (
        spellcaster,
        casting,
    ));
}

fn main() {
    let spells = game::resources::get_spell_list_resource();

    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(game::resources::SpellList(spells))
        .add_systems(Startup, startup)
        .add_systems(Update, game::spellcasting::spell_cast_system)
        .run();
}
