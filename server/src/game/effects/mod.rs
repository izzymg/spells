mod effect_processing;

use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::{
        entity::Entity,
        event::{Event, Events},
    },
};

use super::auras;

/// Queue an effect onto the target
#[derive(Event, Debug, Copy, Clone)]
pub struct EffectQueueEvent {
    pub target: Entity,
    pub health_effect: Option<i64>,
    pub aura_effect: Option<auras::AuraID>,
}

pub struct EffectPlugin;

impl Plugin for EffectPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<Events<EffectQueueEvent>>();
        app.add_systems(FixedUpdate, effect_processing::get_configs());
    }
}
