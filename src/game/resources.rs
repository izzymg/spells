use bevy::ecs::system::Resource;

use crate::game::spells::SpellData;

use super::spells;

#[derive(Resource)]
pub struct SpellList {
    spells: Vec<spells::SpellData>,
}

impl SpellList {
    pub fn get_spell_data(&self, id: usize) -> Option<&SpellData> {
        self.spells.get(id)
    }
}

pub fn get_spell_list_resource() -> SpellList {
    SpellList {
        spells: {
            vec![
                SpellData::new_damage("Fire Ball".into(), 2500, -5),
                SpellData::new_damage("Frost Ball".into(), 4000, -15),
                SpellData::new("Dummy 1".into(), 5,),
            ]
        },
    }
}