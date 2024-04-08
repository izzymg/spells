mod entity_processing;

use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::{
        component::Component,
        entity::Entity,
        event::Event, schedule::IntoSystemConfigs,
    },
};

use super::ServerSets;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<HealthTickEvent>();
        app.add_systems(FixedUpdate, entity_processing::get_config().in_set(ServerSets::EntityProcessing));
    }
}

/// Entity that can die
#[derive(Debug, Component, Default)]
pub struct Health {
    pub hp: i64,
}

impl Health {
    pub fn new(hp: i64) -> Health {
        Health { hp }
    }
}

/// Entity's health should be mutated by hp
#[derive(Debug, Event)]
pub struct HealthTickEvent {
    pub entity: Entity,
    pub hp: i64,
}
