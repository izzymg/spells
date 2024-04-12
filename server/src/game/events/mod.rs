/// general game events
use bevy::prelude::*;

use lib_spells::shared;
/// Queue an effect onto the target
#[derive(Event, Debug, Copy, Clone)]
pub struct EffectQueueEvent {
    pub target: Entity,
    pub health_effect: Option<i64>,
    pub aura_effect: Option<shared::AuraID>,
}

/// `spell_id` should be applied to `target`
#[derive(Clone, Copy, Debug, Event)]
pub struct SpellApplicationEvent {
    pub origin: Entity,
    pub target: Entity,
    pub spell_id: shared::SpellID,
}

/// Request to add an aura child to the given entity
#[derive(Event, Debug)]
pub struct AddAuraEvent {
    pub aura_id: shared::AuraID,
    pub target_entity: Entity,
}

/// Request to drop an aura child from the given entity
#[derive(Event, Debug)]
pub struct RemoveAuraEvent {
    pub aura_id: shared::AuraID,
    pub target_entity: Entity,
}

pub struct GameEventsPlugin;

impl Plugin for GameEventsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<Events<EffectQueueEvent>>() // we want to manually clear this one
            .add_event::<SpellApplicationEvent>()
            .add_event::<AddAuraEvent>()
            .add_event::<RemoveAuraEvent>();
    }
}
