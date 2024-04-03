use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        system::{Commands, In, Query, Res},
    },
    log::*,
    time::{Time, Timer},
};
use std::time::Duration;

use crate::game::alignment::{self, Hostility};

use super::{
    resource,
    spell_application::{self, SpellApplicationEvent},
    SpellID,
};

#[derive(Event)]
pub struct StartCastingEvent {
    pub entity: Entity,
    pub target: Entity,
    pub spell_id: SpellID,
}

impl StartCastingEvent {
    pub fn new(entity: Entity, target: Entity, spell_id: SpellID) -> Self {
        Self {
            entity,
            target,
            spell_id,
        }
    }
}

// Unit is casting a spell
#[derive(Debug, Component)]
pub struct CastingSpell {
    pub spell_id: SpellID,
    pub target: Entity,
    pub cast_timer: Timer,
}

impl CastingSpell {
    fn new(spell_id: SpellID, target: Entity, cast_time: Duration) -> CastingSpell {
        CastingSpell {
            spell_id,
            target,
            cast_timer: Timer::new(cast_time, bevy::time::TimerMode::Once),
        }
    }
}

pub(super) fn check_finished_casts_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CastingSpell, Option<&alignment::FactionMember>)>,
    spell_list: Res<resource::SpellList>,
    faction_checker: alignment::FactionChecker,
    mut spell_app_ev_w: EventWriter<spell_application::SpellApplicationEvent>,
) {
    for (entity, casting, faction_member) in query.iter_mut() {
        if !casting.cast_timer.finished() {
            continue;
        }
        commands.entity(entity).remove::<CastingSpell>();

        let mut push = || {
            // send spell application events
            spell_app_ev_w.send(SpellApplicationEvent {
                origin: entity,
                spell_id: casting.spell_id,
                target: casting.target,
            });
        };

        let spell = spell_list.get_spell_data(casting.spell_id).unwrap();
        let is_selfcast = entity == casting.target;
        if is_selfcast && spell.hostility == Hostility::Friendly {
            // allow friendly self-casts immediately
            push();
        }

        if let Some(caster_faction) = faction_member {
            let target_faction = faction_checker
                .get_entity_faction(casting.target)
                .unwrap_or_default();
            if alignment::is_valid_target(spell.hostility, caster_faction.0, target_faction) {
                // allow valid
                push();
            }
        } else if spell.hostility == Hostility::Hostile {
            // allow hostile when factionless
            push();
        }
    }
}

// Tick spell casts and push finished casts to spell application.
pub(super) fn tick_cast_system(time: Res<Time>, mut query: Query<&mut CastingSpell>) {
    for mut casting in query.iter_mut() {
        casting.cast_timer.tick(time.delta());
    }
}

/// Begin spell casts when event received.
pub(super) fn handle_start_casting_event_system(
    mut events: EventReader<StartCastingEvent>,
    mut commands: Commands,
    spell_list: Res<resource::SpellList>,
) {
    for ev in events.read() {
        if let Some(spell) = spell_list.get_spell_data(ev.spell_id) {
            debug!("{:?} starts casting spell {}", ev.entity, spell.name);
            commands.entity(ev.entity).insert(CastingSpell::new(
                ev.spell_id,
                ev.target,
                spell.cast_time,
            ));
        } else {
            error!("no spell {}", ev.spell_id);
        }
    }
}
