use std::time::Duration;

use bevy::{
    ecs::{
        component::Component, entity::Entity, event::{Event, EventReader, EventWriter}, query::With, system::{Commands, Query, Res}
    },
    log::*,
    time::{Time, Timer},
};

use super::{
    health::{self, HealthTickEvent},
    resources::SpellList,
};

#[derive(Debug, Component)]
pub struct CastingSpell {
    spell_id: usize,
    target: Entity,
    cast_timer: Timer,
}

impl CastingSpell {
    pub fn new(spell_id: usize, target: Entity, cast_time: Duration) -> CastingSpell {
        CastingSpell { spell_id, target, cast_timer: Timer::new(cast_time, bevy::time::TimerMode::Once) }
    }
}


pub fn spell_cast_system(
    mut commands: Commands,
    time: Res<Time>,
    spell_list: Res<SpellList>,
    mut ev_w: EventWriter<health::HealthTickEvent>,
    mut query: Query<(Entity, &mut CastingSpell), With<Spellcaster>>,
) {
    for (entity, mut casting) in query.iter_mut() {
        casting.cast_timer.tick(time.delta());
        debug!(
            "casting spell {} at {} ({}s)",
            casting.spell_id,
            casting.target.index(),
            casting.cast_timer.elapsed_secs()
        );

        if casting.cast_timer.finished() {
            commands.entity(entity).remove::<CastingSpell>();
            cast_spell(
                &mut ev_w,
                &spell_list,
                SpellCastData {
                    caster: entity,
                    target: casting.target,
                    spell_id: casting.spell_id,
                }
            )
        }
    }
}

/// Marks as having spells which can be cast.
/// Contains a list of valid spell IDs in Spellbook.
#[derive(Debug, Component)]
pub struct Spellcaster { }


struct SpellCastData {
    spell_id: usize,
    target: Entity,
    caster: Entity,
}


fn cast_spell(ev_w: &mut EventWriter<health::HealthTickEvent>, spell_list: &Res<SpellList>, data: SpellCastData) {
    if let Some(spell_data) = spell_list.get_spell_data(data.spell_id) {

        // apply target hp
        if let Some(hp) = spell_data.target_health_effect {
            ev_w.send(HealthTickEvent {
                entity: data.target,
                hp,
            });
        }

        // apply self hp
        if let Some(hp) = spell_data.self_health_effect {
            ev_w.send(HealthTickEvent {
                entity: data.caster,
                hp,
            });
        }
    } else {
        error!("no spell at id {}", data.spell_id);
    }
}


#[derive(Event)]
pub struct StartCastingEvent {
    pub entity: Entity,
    pub target: Entity,
    pub spell_id: usize,
}

pub fn start_casting_system(
    mut events: EventReader<StartCastingEvent>,
    mut commands: Commands,
    spell_list: Res<SpellList>
)
{
    for ev in events.read() {
        if let Some(spell) = spell_list.get_spell_data(ev.spell_id) {
            debug!("e{} starts casting spell {}", ev.entity.index(), spell.name);
            commands.entity(ev.entity).insert(CastingSpell::new(ev.spell_id, ev.target, spell.cast_time));
        } else {
            error!("no spell at id {}", ev.spell_id);
        }
    }
}