use bevy::ecs::system::Resource;

use super::SpellData;

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
            SpellData::new_target_hp("Fire Ball".into(), 2500, -55),
            SpellData::new_target_hp("Frost Ball".into(), 4000, -15),
            SpellData::new("Dummy 1".into(), 5,),
        ]
    )
}
