use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res},
    }, log::debug, time::{Time, Timer}
};

use crate::game::health::HealthTickSingle;

use super::resources;

/// Marks as currently casting a spell.
/// Contains an index into a Spellcaster spellbook.
#[derive(Debug, Component)]
pub struct Casting {
    pub spellbook_index: usize,
    pub cast_timer: Timer,
    pub target: Entity,
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
            commands.entity(casting.target).insert(HealthTickSingle(casting_spell.hit_points));
        } else {
            casting.cast_timer.tick(time.delta());
        }
    }
}

pub fn debug_spell_cast_system(
    spell_list: Res<resources::SpellList>,
    mut query: Query<(Entity, &Spellcaster, &Casting)>,
) {
    for (entity, caster, casting) in query.iter_mut() {
        let spell = caster.get_spellbook_spell(casting.spellbook_index);
        let casting_spell = spell_list.get_spell(spell);
        debug!(
            "E{} casting {} -> E{} ({}/{}s)",
            entity.index(),
            casting_spell.name,
            casting.target.index(),
            casting.cast_timer.elapsed_secs(),
            casting_spell.cast_time.as_secs_f32()
        )
    }
}