use std::time::Duration;
use bevy::ecs::system::{Res, Resource, SystemParam};

use super::AuraID;

/// allow us to easily fetch effect data
#[derive(SystemParam)]
pub(super) struct AuraSysResource<'w> {
    weapons: Res<'w, AuraDatabase>,
}

impl<'w> AuraSysResource<'w> {
    pub(super) fn get_status_effect_data(&self, id: AuraID) -> Option<&AuraData> {
        self.weapons.0.get(id.get())
    }
}

/// complex info about a status effect
pub(super) struct AuraData {
    pub name: String,
    pub base_multiplier: i64,
    pub duration: Duration,
    pub status_type: super::StatusEffectType,
}

impl AuraData {
    pub(super) fn new(name: String, base_multiplier: i64, duration: Duration, status_type: super::StatusEffectType) -> AuraData {
        AuraData {
            name,
            base_multiplier,
            duration,
            status_type,
        }
    }
}

// all our complex info about our status effects
#[derive(Resource)]
pub(super) struct AuraDatabase(Vec<AuraData>);

pub(super) fn get_resource() -> AuraDatabase {
    AuraDatabase(vec![
        AuraData::new("Immolated".into(), -5, Duration::from_secs(10), crate::game::auras::StatusEffectType::TickingHP),
        AuraData::new("Arcane Shield".into(), 100, Duration::from_secs(5), crate::game::auras::StatusEffectType::Shield),
    ])
}