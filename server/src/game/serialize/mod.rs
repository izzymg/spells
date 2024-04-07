use bincode;
use serde::Serialize;

use super::spells;

#[derive(Serialize, Debug, Copy, Clone)]
pub struct CasterState {
    pub timer: u128,
    pub max_timer: u128,
    pub spell_id: usize,
}

impl From<&spells::CastingSpell> for CasterState {
    fn from(value: &spells::CastingSpell) -> Self {
        Self {
            timer: value.cast_timer.elapsed().as_millis(),
            max_timer: value.cast_timer.duration().as_millis(),
            spell_id: value.spell_id.get(),
        }
    }
}

#[derive(Default, Debug, Serialize)]
pub struct WorldState {
    pub casters: Vec<CasterState>,
}

impl WorldState {
    pub fn serialize(&self) -> Result<Vec<u8>, bincode::ErrorKind> {
        match bincode::serialize(&self) {
            Ok(data) => Ok(data),
            Err(err) => Err(*err)
        }
    }
}
