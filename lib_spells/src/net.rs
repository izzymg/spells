use crate::shared;
use bevy_ecs::{entity::MapEntities, prelude::*, system::Command};
use bevy_math::*;
use bincode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
pub type SerializationError = bincode::ErrorKind;

#[derive(Debug)]
pub struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error")
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MovementDirection(pub u8);
pub const MOVE_NONE: u8 = 0b00000000;
pub const MOVE_LEFT: u8 = 0b00000001;
pub const MOVE_RIGHT: u8 = 0b00000010;
pub const MOVE_UP: u8 = 0b00000100;
pub const MOVE_DOWN: u8 = 0b00001000;
pub const MOVE_FORWARD: u8 = 0b00010000;
pub const MOVE_BACKWARD: u8 = 0b00100000;

impl From<MovementDirection> for Vec3 {
    fn from(value: MovementDirection) -> Vec3 {
        let mut vec = Vec3::ZERO;
        let dir = value.0;
        if dir & MOVE_LEFT > 0 {
            vec.x += -1.;
        }
        if dir & MOVE_RIGHT > 0 {
            vec.x += 1.;
        }
        if dir & MOVE_UP > 0 {
            vec.y += 1.;
        }
        if dir & MOVE_DOWN > 0 {
            vec.y += -1.;
        }
        if dir & MOVE_FORWARD > 0 {
            vec.z += -1.;
        }
        if dir & MOVE_BACKWARD > 0 {
            vec.z += 1.
        }
        vec
    }
}

impl From<Vec3> for MovementDirection {
    fn from(vec: Vec3) -> Self {
        if vec == Vec3::ZERO {
            return MovementDirection(MOVE_NONE);
        }
        let mut movement = 0_u8;
        if vec.x < 0.0 {
            movement |= MOVE_LEFT;
        } else if vec.x > 0.0 {
            movement |= MOVE_RIGHT;
        }

        if vec.y < 0.0 {
            movement |= MOVE_DOWN;
        } else if vec.y > 0.0 {
            movement |= MOVE_UP;
        }

        if vec.z < 0.0 {
            movement |= MOVE_FORWARD;
        } else if vec.z > 0.0 {
            movement |= MOVE_BACKWARD;
        }

        Self(movement)
    }
}

impl TryFrom<&[u8]> for MovementDirection {
    type Error = ParseError;
    /// Produce a movement direction from a payload.
    fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
        if payload.len() != 1 {
            return Err(ParseError);
        }
        Ok(MovementDirection(u8::from_le_bytes([payload[0]])))
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dir_to_vec() {
        let dir = MovementDirection(MOVE_RIGHT | MOVE_UP | MOVE_DOWN | MOVE_FORWARD);
        let expect = Vec3::new(1.0, 0.0, -1.0);
        assert_eq!(Vec3::from(dir), expect);
        let dir = MovementDirection(MOVE_NONE);
        assert_eq!(Vec3::from(dir), Vec3::ZERO);
    }

    #[test]
    fn test_vec_to_dir() {
        let vec = Vec3::new(1.0, 0.0, -1.0);
        assert!(MovementDirection::from(vec).0 & MOVE_RIGHT > 0);
    }
}
