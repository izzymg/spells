use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res},
    },
    time::{Time, Timer},
};

use super::resources;

/// Marks as currently casting a spell.
/// Contains an index into a Spellcaster spellbook.
#[derive(Debug, Component)]
pub struct Casting {
    pub spellbook_index: usize,
    pub cast_timer: Timer,
}

/// Marks as having spells which can be cast.
/// Contains a list of valid spell IDs in Spellbook.
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
    spell_list: Res<resources::SpellList>,
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