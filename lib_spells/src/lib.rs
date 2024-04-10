pub const CLIENT_EXPECT: &str = "SPELLCLIENT OK 0.1\n";
pub const SERVER_HEADER: &str = "SPELLSERVER 0.1\n";

pub mod serialization {
    use std::collections::HashMap;

    use bincode;
    use serde::{Deserialize, Serialize};

    pub type SerializationError = bincode::ErrorKind;

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

    #[derive(Deserialize, Serialize, Default, Debug, Clone)]
    pub struct EntityState {
        pub auras: Vec<EntityAura>,
        pub health: Option<EntityHealth>,
        pub spell_caster: Option<EntitySpellCaster>,
        pub casting_spell: Option<EntityCastingSpell>,
    }

    impl EntityState {
        pub fn with_aura(mut self, aura: EntityAura) -> Self {
            self.auras.push(aura);
            self
        }

        pub fn with_health(mut self, health: EntityHealth) -> Self {
            self.health = Some(health);
            self
        }

        pub fn with_spell_caster(mut self, caster: EntitySpellCaster) -> Self {
            self.spell_caster = Some(caster);
            self
        }

        pub fn with_casting_spell(mut self, casting: EntityCastingSpell) -> Self {
            self.casting_spell = Some(casting);
            self
        }
    }

    #[derive(Deserialize, Serialize, Debug, Default)]
    pub struct WorldState {
        pub entity_state_map: HashMap<u32, EntityState>,
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

        pub fn update(&mut self, key: u32, new_state: EntityState) {
            if let Some(state) = self.entity_state_map.get(&key) {
                let mut state = state.clone();
                state.auras.extend(new_state.auras);
                state.health = new_state.health.or(state.health);
                state.spell_caster = new_state.spell_caster.or(state.spell_caster);
                state.casting_spell = new_state.casting_spell.or(state.casting_spell);
                self.entity_state_map.insert(key, state);
            } else {
                self.entity_state_map.insert(key, new_state);
            }
        }
    }
}
