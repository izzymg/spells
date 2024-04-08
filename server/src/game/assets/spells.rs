use bevy::ecs::system::Resource;
use core::fmt;
use std::time::Duration;

use crate::game::{assets, components};

/// We can use this to look up complex data about a spell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpellID(usize);

impl SpellID {
    pub fn get(self) -> usize {
        self.0
    }
}

impl From<usize> for SpellID {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl fmt::Display for SpellID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(SPELL:{})", self.0)
    }
}

/// Database of spells data by `SpellID`
#[derive(Default, Debug)]
pub struct SpellData {
    pub name: String,
    pub cast_time: Duration,
    pub hostility: components::Hostility,
    pub target_health_effect: Option<i64>,
    pub target_aura_effect: Option<assets::AuraID>,
}

impl SpellData {
    pub fn new(name: String, cast_ms: u64) -> Self {
        Self {
            name: name,
            cast_time: Duration::from_millis(cast_ms),
            ..Default::default()
        }
    }

    pub fn with_target_hp(mut self, hp: i64) -> Self {
        self.target_health_effect = Some(hp);
        self
    }

    pub fn with_target_aura(mut self, aura: assets::AuraID) -> Self {
        self.target_aura_effect = Some(aura);
        self
    }

    pub fn mark_friendly(mut self) -> Self {
        self.hostility = components::Hostility::Friendly;
        self
    }
}

#[derive(Resource)]
pub struct SpellList(pub Vec<SpellData>);

impl SpellList {
    pub fn get_spell_data(&self, id: SpellID) -> Option<&SpellData> {
        self.0.get(id.get())
    }
}

pub(super) fn get_spell_list_resource() -> SpellList {
    SpellList(vec![
        SpellData::new("Fire Ball".into(), 500)
            .with_target_hp(-50)
            .with_target_aura(0.into()),
        SpellData::new("Arcane Barrier".into(), 0)
            .with_target_aura(1.into())
            .mark_friendly(),
    ])
}
