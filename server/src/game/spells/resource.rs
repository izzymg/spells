use std::time::Duration;
use bevy::ecs::system::Resource;

use crate::game::{alignment::{self, Hostility}, auras};

use super::SpellID;

#[derive(Default, Debug)]
pub(super) struct SpellData {
    pub name: String,
    pub cast_time: Duration,
    pub hostility: alignment::Hostility,
    pub target_health_effect: Option<i64>,
    pub target_aura_effect: Option<auras::AuraID>,
}

impl SpellData {
    fn new(name: String, cast_ms: u64) -> Self {
        Self {
            name: name,
            cast_time: Duration::from_millis(cast_ms),
            ..Default::default()
        }
    }
    
    fn with_target_hp(mut self, hp: i64) -> Self {
        self.target_health_effect = Some(hp);
        self
    }

    fn with_target_aura(mut self, aura: auras::AuraID) -> Self {
        self.target_aura_effect = Some(aura);
        self
    }

    fn mark_friendly(mut self) -> Self {
        self.hostility = Hostility::Friendly;
        self
    }

}

#[derive(Resource)]
pub(super) struct SpellList(pub Vec<SpellData>);

impl SpellList {
    pub(super) fn get_spell_data(&self, id: SpellID) -> Option<&SpellData> {
        self.0.get(id.get())
    }
}

pub(super) fn get_spell_list_resource() -> SpellList {
    SpellList(
        vec![
            SpellData::new("Fire Ball".into(), 52500).with_target_hp(-50).with_target_aura(0.into()),
            SpellData::new("Arcane Barrier".into(), 0).with_target_aura(1.into()).mark_friendly(),
        ]
    )
}
