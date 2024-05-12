pub mod packet;
use crate::shared;
use bevy_ecs::{entity::MapEntities, prelude::*, system::Command};
use bevy_math::*;
use bincode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
pub type SerializationError = bincode::ErrorKind;

// we making it into the mental asylum with this one
// this just generates our serializable entity state struct so we don't have 1000 fields and update/from implementations
// could replace this with a proc_macro that so we can derive(stateable) impl or something too
macro_rules! gen_state {
    ( $($t:ty, $field:ident),* ) => {
        /// State for an entity we care to replicate
        #[derive(Default, Debug, Clone, Serialize, Deserialize)]
        pub struct EntityState {
            $ ( pub $field: Option<$t> ),*
        }
        impl EntityState {
            /// Merges this state with `other`, prioritising `Some` values on `other`
            pub fn update(mut self, other: Self) -> Self {
                $( self.$field = other.$field.or(self.$field);  )*
                self
            }
        }
        $ (
        impl From<$t> for EntityState {
            fn from(value: $t) -> Self {
                Self {
                    $field: Some(value),
                    ..Default::default()
                }
            }
        })*

        impl Command for AddEntityStateCommand {
            fn apply(self, world: &mut World) {
                $(
                    if let Some(c) = self.entity_state.$field {
                        world.get_entity_mut(self.entity).unwrap().insert(c.clone());
                    }
                )*
            }
        }

        pub fn query_world_state(world: &mut World) -> WorldState {
            let mut state = WorldState::default();
            $(
                let mut query = world.query::<(Entity, &$t)>();
                for (entity, comp) in query.iter(&world) {
                    state.update(entity, comp.clone().into());
                }
            )*
            state
        }
    }
}

macro_rules! state_map_entities {
    ( $($field:ident),* ) => {
        impl MapEntities for EntityState {
            fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
                $(
                    if let Some(f) = self.$field.as_mut() {
                        f.map_entities(entity_mapper);
                    }
                )*
            }
        }
    }
}

pub struct AddEntityStateCommand {
    pub entity: Entity,
    pub entity_state: EntityState,
}

// the actual net state for every entity
gen_state!(
    shared::Health,
    health,
    shared::SpellCaster,
    spellcaster,
    shared::Aura,
    aura,
    shared::CastingSpell,
    casting_spell,
    shared::Position,
    position,
    shared::Player,
    player,
    shared::Name,
    name,
    shared::Velocity,
    velocity
);

state_map_entities!(aura);

/// Maps a set of entities to their component state for network magic.
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct WorldState {
    pub entity_state_map: HashMap<Entity, EntityState>,
}

impl WorldState {
    /// Push `new_state` into the map, calling `update` on the existing state if it exists
    pub fn update(&mut self, key: Entity, new_state: EntityState) {
        if let Some(existing) = self.entity_state_map.get(&key) {
            let existing = existing.clone().update(new_state);
            self.entity_state_map.insert(key, existing);
        } else {
            self.entity_state_map.insert(key, new_state);
        }
    }
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Debug, Copy, Clone)]
pub struct ClientInfo {
    pub you: Entity,
}

pub fn serialize<T: Serialize>(data: &T) -> Result<Vec<u8>, SerializationError> {
    match bincode::serialize(data) {
        Ok(data) => Ok(data),
        Err(err) => Err(*err),
    }
}

pub fn deserialize<'a, T: Deserialize<'a>>(data: &'a [u8]) -> Result<T, SerializationError> {
    match bincode::deserialize::<'a, T>(data) {
        Ok(state) => Ok(state),
        Err(e) => Err(*e),
    }
}

