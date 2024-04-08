use bevy::ecs::entity::Entity;
use bincode;
use serde::Serialize;

#[derive(Serialize, Debug, Copy, Clone)]
pub struct EntityCaster {
    pub entity: Entity,
    pub timer: u128,
    pub max_timer: u128,
    pub spell_id: usize,
}

#[derive(Serialize, Debug, Copy, Clone)]
pub struct EntityHealth {
    pub entity: Entity,
    pub health: i64,
}

#[derive(Serialize, Debug, Copy, Clone)]
pub struct EntityAura {
    pub entity: Entity,
    pub aura_id: usize,
    pub remaining: u128,
}

#[derive(Debug, Serialize, Default)]
pub struct WorldState {
    pub health: Vec<EntityHealth>,
    pub casters: Vec<EntityCaster>,
    pub auras: Vec<EntityAura>,
}

impl WorldState {
    pub fn serialize(&self) -> Result<Vec<u8>, bincode::ErrorKind> {
        match bincode::serialize(&self) {
            Ok(data) => Ok(data),
            Err(err) => Err(*err),
        }
    }
}
