/// general game events
use bevy::prelude::*;

use super::assets;
/// Queue an effect onto the target
#[derive(Event, Debug, Copy, Clone)]
pub struct EffectQueueEvent {
    pub target: Entity,
    pub health_effect: Option<i64>,
    pub aura_effect: Option<assets::AuraID>,
}

/// `spell_id` should be applied to `target`
#[derive(Clone, Copy, Debug, Event)]
pub struct SpellApplicationEvent {
    pub origin: Entity,
    pub target: Entity,
    pub spell_id: assets::SpellID,
}

/// Unit should start casting `spell_id` at `target`
#[derive(Event)]
pub struct StartCastingEvent {
    pub entity: Entity,
    pub target: Entity,
    pub spell_id: assets::SpellID,
}

impl StartCastingEvent {
    pub fn new(entity: Entity, target: Entity, spell_id: assets::SpellID) -> Self {
        Self {
            entity,
            target,
            spell_id,
        }
    }
}

/// Request to add an aura child to the given entity
#[derive(Event, Debug)]
pub struct AddAuraEvent {
    pub aura_id: assets::AuraID,
    pub target_entity: Entity,
}

/// Request to drop an aura child from the given entity
#[derive(Event, Debug)]
pub struct RemoveAuraEvent {
    pub aura_id: assets::AuraID,
    pub target_entity: Entity,
}

pub struct GameEventsPlugin;

impl Plugin for GameEventsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .init_resource::<Events<EffectQueueEvent>>() // we want to manually clear this one
            .add_event::<StartCastingEvent>()
            .add_event::<SpellApplicationEvent>()
            .add_event::<AddAuraEvent>()
            .add_event::<RemoveAuraEvent>();
    }
}
