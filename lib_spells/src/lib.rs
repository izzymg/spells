pub const CLIENT_EXPECT: &str = "SPELLCLIENT OK 0.1\n";
pub const SERVER_HEADER: &str = "SPELLSERVER 0.1\n";

pub mod alignment;

pub mod shared {
    use core::fmt;
    use std::{collections::HashMap, time::Duration};

    use bevy::prelude::*;
    use bincode;
    use serde::{Deserialize, Serialize};

    pub type SerializationError = bincode::ErrorKind;

    /// Entity can be harmed and healed
    #[derive(Deserialize, Serialize, Component, Debug, Copy, Clone)]
    pub struct Health(pub i64);

    /// Represents one aura belonging to the parent of this entity
    #[derive(Deserialize, Serialize, Component, Debug, Clone)]
    pub struct Aura {
        pub id: AuraID,
        pub duration: Timer,
        pub owner: Entity,
    }

    impl Aura {
        pub fn get_remaining_time(&self) -> Duration {
            self.duration.duration() - self.duration.elapsed()
        }
    }

    /// Used to look up an aura in the aura list resource
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct AuraID(usize);

    impl AuraID {
        pub fn get(self) -> usize {
            self.0
        }
    }

    impl From<usize> for AuraID {
        fn from(value: usize) -> Self {
            Self(value)
        }
    }

    impl fmt::Display for AuraID {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "(AURA:{})", self.0)
        }
    }

    /// Possible aura types
    pub enum AuraType {
        TickingHP,
        Shield,
    }

    /// We can use this to look up complex data about a spell
    #[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SpellID(usize);

    impl SpellID {
        pub fn get(self) -> usize {
            self.0
        }
    }

    impl From<usize> for SpellID {
        fn from(value: usize) -> Self {
            Self(value)
        }
    }

    impl fmt::Display for SpellID {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "(SPELL:{})", self.0)
        }
    }

    /// Unit can cast spells
    #[derive(Debug, Component, Copy, Clone, Serialize, Deserialize)]
    pub struct SpellCaster;

    /// Unit is casting a spell
    #[derive(Debug, Component, Clone, Serialize, Deserialize)]
    pub struct CastingSpell {
        pub spell_id: SpellID,
        pub target: Entity,
        pub cast_timer: Timer,
    }

    impl CastingSpell {
        pub fn new(spell_id: SpellID, target: Entity, cast_time: Duration) -> CastingSpell {
            CastingSpell {
                spell_id,
                target,
                cast_timer: Timer::new(cast_time, bevy::time::TimerMode::Once),
            }
        }
    }

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
        Health,
        health,
        SpellCaster,
        spellcaster,
        Aura,
        aura,
        CastingSpell,
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
}
