/// general game events

use bevy::{app::Plugin, ecs::{entity::Entity, event::Event}};

use super::{auras, spells};
/// Queue an effect onto the target
#[derive(Event, Debug, Copy, Clone)]
pub struct EffectQueueEvent {
    pub target: Entity,
    pub health_effect: Option<i64>,
    pub aura_effect: Option<auras::AuraID>,
}


/// `spell_id` should be applied to `target`
#[derive(Clone, Copy, Debug, Event)]
pub struct SpellApplicationEvent {
    pub origin: Entity,
    pub target: Entity,
    pub spell_id: spells::SpellID,
}


/// Unit should start casting `spell_id` at `target`
#[derive(Event)]
pub struct StartCastingEvent {
    pub entity: Entity,
    pub target: Entity,
    pub spell_id: spells::SpellID,
}

impl StartCastingEvent {
    pub fn new(entity: Entity, target: Entity, spell_id: spells::SpellID) -> Self {
        Self {
            entity,
            target,
            spell_id,
        }
    }
}

pub struct GameEventsPlugin;

impl Plugin for GameEventsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<StartCastingEvent>();
        app.add_event::<SpellApplicationEvent>();
        app.add_event::<EffectQueueEvent>();
    }
}