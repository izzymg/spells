use bevy::{ecs::system::SystemParam, prelude::*};
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

// all our complex info about our status effects
#[derive(Resource)]
pub struct AuraDatabase(pub Vec<AuraData>);

pub(super) fn get_auras_resource() -> AuraDatabase {
    AuraDatabase(vec![
        AuraData::new(
            "Immolated".into(),
            -5,
            Duration::from_secs(10),
            AuraType::TickingHP,
        ),
        AuraData::new(
            "Arcane Shield".into(),
            100,
            Duration::from_secs(5),
            AuraType::Shield,
        ),
    ])
}
