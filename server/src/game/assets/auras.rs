use bevy::prelude::*;
use lib_spells::shared;
use std::time::Duration;

/// Complex info about a status effect
pub struct AuraData {
    pub name: String,
    pub base_multiplier: i64,
    pub duration: Duration,
    pub status_type: shared::AuraType,
}

impl AuraData {
    pub fn new(
        name: String,
        base_multiplier: i64,
        duration: Duration,
        status_type: shared::AuraType,
    ) -> AuraData {
        AuraData {
            name,
            base_multiplier,
            duration,
            status_type,
        }
    }
}

/// Maps aura IDs to aura data.
#[derive(Resource)]
pub struct AurasAsset(pub Vec<AuraData>);

impl AurasAsset {
    pub fn lookup(&self, aura_id: shared::AuraID) -> Option<&AuraData> {
        self.0.get(aura_id.get())
    }
}

pub(super) fn get_auras_resource() -> AurasAsset {
    AurasAsset(vec![
        AuraData::new(
            "Immolated".into(),
            -5,
            Duration::from_secs(10),
            shared::AuraType::TickingHP,
        ),
        AuraData::new(
            "Arcane Shield".into(),
            100,
            Duration::from_secs(7),
            shared::AuraType::Shield,
        ),
    ])
}
