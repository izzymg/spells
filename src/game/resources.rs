use std::time::Duration;
use bevy::ecs::system::Resource;

use super::spells;

#[derive(Resource)]
pub struct SpellList(pub Vec<spells::Spell>);

pub fn get_spell_list_resource() -> SpellList {
    SpellList(vec![
        spells::Spell {
            cast_time: Duration::from_millis(2000),
            hit_points: -15,
            name: String::from("Fireball"),
        },
        spells::Spell {
            cast_time: Duration::from_millis(2500),
            hit_points: -15,
            name: String::from("Ice Crush"),
        },
    ])
}

impl SpellList {
    pub fn get_spell(&self, i: usize) -> Option<&spells::Spell> {
        self.0.get(i) 
    }
}
