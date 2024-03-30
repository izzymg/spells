use bevy::ecs::system::Resource;

use crate::game::{auras::AuraData, spells::SpellData};

use super::{auras, spells};

#[derive(Resource)]
pub struct SpellList(pub Vec<spells::SpellData>);

impl SpellList {
    pub fn get_spell_data(&self, id: usize) -> Option<&SpellData> {
        self.0.get(id)
    }
}

pub fn get_spell_list_resource() -> SpellList {
    SpellList(
        vec![
            SpellData::new_target_hp("Fire Ball".into(), 2500, -5),
            SpellData::new_target_hp("Frost Ball".into(), 4000, -15),
            SpellData::new("Dummy 1".into(), 5,),
        ]
    )
}

#[derive(Resource)]
pub struct AuraList(pub Vec<auras::AuraData>);

impl AuraList {
    pub fn get_aura_data(&self, id: usize) -> Option<&AuraData> {
        self.0.get(id)
    }
}

pub fn get_aura_list_resource() -> AuraList {
    AuraList(
            vec![
                AuraData::new("Burning".into(), 2000, auras::AuraType::THORNS),
                AuraData::new("Burning".into(), 50000, auras::AuraType::SHIELD),
            ]
    )
}
