use std::time::Duration;
use bevy::{ecs::{
        component::Component, entity::Entity, event::{Event, EventReader, EventWriter}, system::{Commands, Query, Res}
    }, log::*, time::{Time, Timer}
};

use super::{resource, spell_application, SpellID};

#[derive(Event)]
pub struct StartCastingEvent {
    pub entity: Entity,
    pub target: Entity,
    pub spell_id: SpellID,
}

impl StartCastingEvent {
    pub fn new(entity: Entity, target: Entity, spell_id: SpellID) -> Self {
        Self {
            entity, target, spell_id,
        }
    }
}

// Unit is casting a spell
#[derive(Debug, Component)]
pub(super) struct CastingSpell {
    spell_id: SpellID,
    target: Entity,
    cast_timer: Timer,
}

impl CastingSpell {
    fn new(spell_id: SpellID, target: Entity, cast_time: Duration) -> CastingSpell {
        CastingSpell { spell_id, target, cast_timer: Timer::new(cast_time, bevy::time::TimerMode::Once) }
    }
}

// Tick spell casts and push finished casts to spell application.
pub(super) fn spellcast_tick_system(
    mut commands: Commands,
    time: Res<Time>,
    mut spell_app_ev_w: EventWriter<spell_application::SpellApplicationEvent>,
    mut query: Query<(Entity, &mut CastingSpell)>,
) {
    for (entity, mut casting) in query.iter_mut() {
        casting.cast_timer.tick(time.delta());
        debug!(
            "spell cast {} at {:?} ({}s)",
            casting.spell_id,
            casting.target,
            casting.cast_timer.elapsed_secs()
        );

        if casting.cast_timer.finished() {
            commands.entity(entity).remove::<CastingSpell>();

            // todo: THIS -> SPELL CREATION -> SPELL APPLICATION
            // for projectile casting etc
            spell_app_ev_w.send(spell_application::SpellApplicationEvent {
                origin: entity,
                target: casting.target,
                spell_id: casting.spell_id,
            });
        }
    }
}


/// Begin spell casts when event received.
pub(super) fn handle_start_casting_event_system(
    mut events: EventReader<StartCastingEvent>,
    mut commands: Commands,
    spell_list: Res<resource::SpellList>
)
{
    for ev in events.read() {
        if let Some(spell) = spell_list.get_spell_data(ev.spell_id) {
            debug!("{:?} starts casting spell {}", ev.entity, spell.name);
            commands.entity(ev.entity).insert(CastingSpell::new(ev.spell_id, ev.target, spell.cast_time));
        } else {
            error!("no spell {}", ev.spell_id);
        }
    }
}