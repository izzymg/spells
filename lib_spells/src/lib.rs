pub const CLIENT_EXPECT: &str = "SPELLCLIENT OK 0.1\n";
pub const SERVER_HEADER: &str = "SPELLSERVER 0.1\n";

pub mod serialization {
    use std::collections::HashMap;

    use bevy::prelude::*;
    use bincode;
    use serde::{Deserialize, Serialize};

    pub type SerializationError = bincode::ErrorKind;

    /// Entity can be harmed and healed
    #[derive(Deserialize, Serialize, Component, Debug, Copy, Clone)]
    pub struct Health(pub i64);

    /// Entity is a shadow wizard money gang member
    #[derive(Deserialize, Serialize, Component, Debug, Copy, Clone)]
    pub struct SpellCaster;

    #[derive(Deserialize, Serialize, Debug, Copy, Clone)]
    pub struct EntityCastingSpell {
        pub timer: u64,
        pub max_timer: u64,
        pub spell_id: usize,
    }

    #[derive(Deserialize, Serialize, Debug, Copy, Clone)]
    pub struct EntitySpellCaster;

    #[derive(Deserialize, Serialize, Debug, Copy, Clone)]
    pub struct EntityHealth {
        pub health: i64,
    }

    #[derive(Deserialize, Serialize, Debug, Copy, Clone)]
    pub struct EntityAura {
        pub aura_id: usize,
        pub remaining: u64,
    }

    // we making it into the mental asylum with this one
    macro_rules! gen_state {
        ( $($t:ty, $field:ident),* ) => {
            #[derive(Default, Debug, Clone, Serialize, Deserialize)]
            pub struct NeoState {
                $ ( $field: Option<$t> ),*
            }
            impl NeoState {
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
    gen_state!(Health, health, SpellCaster, spellcaster);
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
