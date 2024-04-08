mod effect_processing;

use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::{
        entity::Entity,
        event::{Event, Events},
    },
};

use crate::game::{auras, events};
pub struct EffectPlugin;

impl Plugin for EffectPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<Events<events::EffectQueueEvent>>();
        app.add_systems(FixedUpdate, effect_processing::get_configs());
    }
}
