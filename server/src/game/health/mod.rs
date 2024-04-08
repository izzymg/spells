mod entity_processing;

use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::{
        component::Component,
        schedule::IntoSystemConfigs,
    },
};

use super::ServerSets;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(FixedUpdate, entity_processing::get_config().in_set(ServerSets::EntityProcessing));
    }
}

/// Entity that can die
#[derive(Debug, Component, Default)]
pub struct Health {
    pub hp: i64,
}