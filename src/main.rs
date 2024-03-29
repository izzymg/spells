use bevy::prelude::*;
use resources::SpellList;
use spells::{Casting, Spellcaster};
pub mod resources;

mod spells {
    use std::time::Duration;

    use bevy::{
        ecs::{
            component::Component,
            entity::Entity,
            system::{Commands, Query, Res},
        },
        time::{Time, Timer},
    };

    use crate::resources::SpellList;

    #[derive(Debug)]
    pub struct Spell {
        pub name: String,
        pub hit_points: i64,
        pub cast_time: Duration,
    }

    #[derive(Debug, Component)]
    pub struct Casting {
        pub spellbook_index: usize,
        pub cast_timer: Timer,
    }

    #[derive(Debug, Component)]
    pub struct Spellcaster {
        pub spellbook: Vec<usize>,
    }

    impl Spellcaster {
        // todo: add error
        pub fn get_spellbook_spell(&self, i: usize) -> usize {
            self.spellbook[i]
        }
    }

    pub fn spell_cast_system(
        mut commands: Commands,
        spell_list: Res<SpellList>,
        time: Res<Time>,
        mut query: Query<(Entity, &Spellcaster, &mut Casting)>,
    ) {
        for (entity, caster, mut casting) in query.iter_mut() {
            let spell = caster.get_spellbook_spell(casting.spellbook_index);

            let casting_spell = spell_list.get_spell(spell);

            if casting.cast_timer.finished() {
                commands.entity(entity).remove::<Casting>();
                println!("spell cast system: CASTED: {}", casting_spell.name)
            } else {
                casting.cast_timer.tick(time.delta());
                println!(
                    "spell cast system: CASTING: {} {}",
                    casting_spell.name,
                    casting.cast_timer.elapsed_secs()
                )
            }
        }
    }
}

mod health {
    use bevy::ecs::component::Component;

    #[derive(Debug, Component)]
    pub struct Damageable {
        pub hit_points: i64,
    }
}

fn startup(mut commands: Commands, spell_list: Res<SpellList>) {

    let spellcaster = Spellcaster {
        spellbook: [0, 1].to_vec(),
    };

    let casting_spell = spell_list.get_spell(
        spellcaster.get_spellbook_spell(0),
    );

    println!("creating spellcaster to cast {} for {:?}", casting_spell.name, casting_spell.cast_time);

    let casting = Casting {
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
    let spells = resources::get_spell_list_resource();

    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(resources::SpellList(spells))
        .add_systems(Startup, startup)
        .add_systems(Update, spells::spell_cast_system)
        .run();
}
