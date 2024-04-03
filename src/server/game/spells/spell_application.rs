use bevy::{
    ecs::{
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        system::Res,
    },
    log,
};

use crate::game::effect_application;

use super::{resource::SpellList, SpellID};

/// handles application of spell effects to a target
/// fetches relevant damage data & translates them into effect application events

#[derive(Clone, Copy, Debug, Event)]
pub(super) struct SpellApplicationEvent {
    pub origin: Entity,
    pub target: Entity,
    pub spell_id: SpellID,
}

pub(super) fn handle_spell_applications_system(
    spell_list: Res<SpellList>,
    mut effect_ev_w: EventWriter<effect_application::EffectQueueEvent>,
    mut spell_ev_r: EventReader<SpellApplicationEvent>,
) {
    for ev in spell_ev_r.read() {
        if let Some(spell_data) = spell_list.get_spell_data(ev.spell_id) {
            effect_ev_w.send(effect_application::EffectQueueEvent {
                target: ev.target,
                health_effect: spell_data.target_health_effect,
                aura_effect: spell_data.target_aura_effect,
            });
        } else {
            log::warn!("no spell {}", ev.spell_id);
        }
    }
}
