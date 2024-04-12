use bevy::ecs::system::Resource;
use std::time::Duration;

use lib_spells::{alignment, shared};

/// Database of spells data by `SpellID`
#[derive(Default, Debug)]
pub struct SpellData {
    pub name: String,
    pub cast_time: Duration,
    pub hostility: alignment::Hostility,
    pub target_health_effect: Option<i64>,
    pub target_aura_effect: Option<shared::AuraID>,
}

impl SpellData {
    pub fn new(name: String, cast_ms: u64) -> Self {
        Self {
            name,
            cast_time: Duration::from_millis(cast_ms),
            ..Default::default()
        }
    }

    pub fn with_target_hp(mut self, hp: i64) -> Self {
        self.target_health_effect = Some(hp);
        self
    }

    pub fn with_target_aura(mut self, aura: shared::AuraID) -> Self {
        self.target_aura_effect = Some(aura);
        self
    }

    pub fn mark_friendly(mut self) -> Self {
        self.hostility = alignment::Hostility::Friendly;
        self
    }
}

#[derive(Resource)]
pub struct SpellsAsset(pub Vec<SpellData>);

impl SpellsAsset {
    pub fn get_spell_data(&self, id: shared::SpellID) -> Option<&SpellData> {
        self.0.get(id.get())
    }
}

pub(super) fn get_spell_list_resource() -> SpellsAsset {
    SpellsAsset(vec![
        SpellData::new("Fire Ball".into(), 5500)
            .with_target_hp(-50)
            .with_target_aura(0.into()),
        SpellData::new("Grand Heal".into(), 5500).with_target_hp(40),
        SpellData::new("Arcane Barrier".into(), 0)
            .with_target_aura(1.into())
            .mark_friendly(),
    ])
}
