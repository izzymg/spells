use bevy::prelude::*;
use core::fmt;
use std::time::Duration;

/// Used to look up an aura in the aura list resource.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuraID(usize);

impl AuraID {
    fn get(self) -> usize {
        self.0
    }
}

impl From<usize> for AuraID {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl fmt::Display for AuraID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(AURA:{})", self.0)
    }
}

/// Possible aura types
pub enum AuraType {
    TickingHP,
    Shield,
}

/// Complex info about a status effect
pub struct AuraData {
    pub name: String,
    pub base_multiplier: i64,
    pub duration: Duration,
    pub status_type: AuraType,
}

impl AuraData {
    pub fn new(
        name: String,
        base_multiplier: i64,
        duration: Duration,
        status_type: AuraType,
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
    pub fn lookup(&self, aura_id: AuraID) -> Option<&AuraData> {
        self.0.get(aura_id.get())
    }
}

pub(super) fn get_auras_resource() -> AurasAsset {
    AurasAsset(vec![
        AuraData::new(
            "Immolated".into(),
            -5,
            Duration::from_secs(10),
            AuraType::TickingHP,
        ),
        AuraData::new(
            "Arcane Shield".into(),
            100,
            Duration::from_secs(7),
            AuraType::Shield,
        ),
    ])
}