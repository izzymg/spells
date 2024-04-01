use std::time::Duration;

use bevy::{
    ecs::{
        system::{Res, Resource, SystemParam},
    },
};

/// allow us to easily fetch effect data
#[derive(SystemParam)]
pub(super) struct StatusEffectSystem<'w> {
    weapons: Res<'w, StatusEffectDatabase>,
}

impl<'w> StatusEffectSystem<'w> {
    pub(super) fn get_status_effect_data(&self, id: usize) -> Option<&StatusEffectData> {
        self.weapons.0.get(id)
    }
}

/// complex info about a status effect
pub(super) struct StatusEffectData {
    pub name: String,
    pub base_multiplier: i64,
    pub duration: Duration,
    pub status_type: super::StatusEffectType,
}

impl StatusEffectData {
    pub(super) fn new(name: String, base_multiplier: i64, duration: Duration, status_type: super::StatusEffectType) -> StatusEffectData {
        StatusEffectData {
            name,
            base_multiplier,
            duration,
            status_type,
        }
    }
}

// all our complex info about our status effects
#[derive(Resource)]
pub(super) struct StatusEffectDatabase(Vec<StatusEffectData>);

pub(super) fn get_resource() -> StatusEffectDatabase {
    StatusEffectDatabase(vec![
        StatusEffectData::new("Immolated".into(), 5, Duration::from_secs(5), super::StatusEffectType::TickingHP),
        StatusEffectData::new("Arcane Shield".into(), 1, Duration::from_secs(2), super::StatusEffectType::Shield),
    ])
}