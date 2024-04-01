// shields a target from damage

use bevy::{ecs::{component::Component, query::With, system::Query}, log};

#[derive(Component)]
pub(super) struct StatusShield;

impl StatusShield {
    pub fn new() -> StatusShield {
        StatusShield
    }
}

// do something for shield systems
pub(super) fn status_shield_system(
    query: Query<&super::StatusEffect, With<StatusShield>>,
    status_system: super::resource::StatusEffectSystem
) {
    for effect in query.iter() {
        if let Some(status_effect) = status_system.get_status_effect_data(effect.id) {
            log::debug!("i'm shielded {}", status_effect.name);
        }
    }
}