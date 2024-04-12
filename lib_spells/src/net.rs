use std::collections::HashMap;

use bevy::prelude::*;
use bincode;
use serde::{Deserialize, Serialize};

use crate::shared;

pub type SerializationError = bincode::ErrorKind;
// we making it into the mental asylum with this one
// this just generates our serializable entity state struct so we don't have 1000 fields and update/from implementations
// could replace this with a proc_macro that so we can derive(stateable) impl or something too
macro_rules! gen_state {
        ( $($t:ty, $field:ident),* ) => {
            /// State for an entity we care to replicate
            #[derive(Default, Debug, Clone, Serialize, Deserialize)]
            pub struct NeoState {
                $ ( $field: Option<$t> ),*
            }
            impl NeoState {
                /// Merges this state with `other`, prioritising `Some` values on `other`
                pub fn update(mut self, other: Self) -> Self {
                    $( self.$field = other.$field.or(self.$field);  )*
                    self
                }
            }
            $ (
            impl From<$t> for NeoState {
                fn from(value: $t) -> Self {
                    Self {
                        $field: Some(value),
                        ..default()
                    }
                }
            })*
        }
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
    casting_spell
);

/// Maps a set of entities to their component state for network magic.
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct WorldState {
    pub entity_state_map: HashMap<u32, NeoState>,
}

impl WorldState {
    pub fn serialize(&self) -> Result<Vec<u8>, SerializationError> {
        match bincode::serialize(&self) {
            Ok(data) => Ok(data),
            Err(err) => Err(*err),
        }
    }

    pub fn deserialize(data: &[u8]) -> Result<WorldState, SerializationError> {
        match bincode::deserialize::<WorldState>(data) {
            Ok(state) => Ok(state),
            Err(e) => Err(*e),
        }
    }

    /// Push `new_state` into the map, calling `update` on the existing state if it exists
    pub fn update(&mut self, key: u32, new_state: NeoState) {
        if let Some(existing) = self.entity_state_map.get(&key) {
            let existing = existing.clone().update(new_state);
            self.entity_state_map.insert(key, existing);
        } else {
            self.entity_state_map.insert(key, new_state);
        }
    }
}
