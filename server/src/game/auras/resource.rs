use std::time::Duration;
use bevy::ecs::system::{Res, Resource, SystemParam};

use super::AuraID;

/// allow us to easily fetch effect data
#[derive(SystemParam)]
pub struct AuraSysResource<'w> {
    weapons: Res<'w, AuraDatabase>,
}

impl<'w> AuraSysResource<'w> {
    pub fn get_status_effect_data(&self, id: AuraID) -> Option<&AuraData> {
        self.weapons.0.get(id.get())
    }
}

/// complex info about a status effect
pub struct AuraData {
    pub name: String,
    pub base_multiplier: i64,
    pub duration: Duration,
    pub status_type: super::AuraType,
}

impl AuraData {
    pub fn new(name: String, base_multiplier: i64, duration: Duration, status_type: super::AuraType) -> AuraData {
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
pub struct AuraDatabase(pub Vec<AuraData>);

pub fn get_resource() -> AuraDatabase {
    AuraDatabase(vec![
        AuraData::new("Immolated".into(), -5, Duration::from_secs(10), crate::game::auras::AuraType::TickingHP),
        AuraData::new("Arcane Shield".into(), 100, Duration::from_secs(5), crate::game::auras::AuraType::Shield),
    ])
}