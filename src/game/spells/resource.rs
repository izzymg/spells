use std::time::Duration;

use bevy::ecs::system::Resource;

#[derive(Default, Debug)]
pub(super) struct SpellData {
    pub name: String,
    pub cast_time: Duration,
    pub target_health_effect: Option<i64>,
    pub self_health_effect: Option<i64>,
    pub self_aura_effect: Option<usize>,
    pub target_aura_effect: Option<usize>,
}

impl SpellData {
    fn new(name: String, cast_ms: u64) -> Self {
        Self {
            name: name,
            cast_time: Duration::from_millis(cast_ms),
            self_health_effect: None,
            target_health_effect: None,
            self_aura_effect: None,
            target_aura_effect: None,
        }
    }
    
    fn with_target_hp(mut self, hp: i64) -> Self {
        self.target_health_effect = Some(hp);
        self
    }

    fn with_self_hp(mut self, hp: i64) -> Self {
        self.target_health_effect = Some(hp);
        self
    }

    fn with_self_aura(mut self, aura: usize) -> Self {
        self.self_aura_effect = Some(aura);
        self
    }

    fn with_target_aura(mut self, aura: usize) -> Self {
        self.target_aura_effect = Some(aura);
        self
    }
}


#[derive(Resource)]
pub(super) struct SpellList(pub Vec<SpellData>);

impl SpellList {
    pub(super) fn get_spell_data(&self, id: usize) -> Option<&SpellData> {
        self.0.get(id)
    }
}

pub(super) fn get_spell_list_resource() -> SpellList {
    SpellList(
        vec![
            SpellData::new("Fire Ball".into(), 2500).with_target_hp(-15).with_target_aura(0),
            SpellData::new("Arcane Barrier".into(), 500).with_self_aura(1),
            SpellData::new("Restore Soul".into(), 5000).with_self_hp(50)
        ]
    )
}
