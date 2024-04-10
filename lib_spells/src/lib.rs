pub const CLIENT_EXPECT: &str = "SPELLCLIENT OK 0.1\n";
pub const SERVER_HEADER: &str = "SPELLSERVER 0.1\n";

pub mod serialization {
    use bincode;
    use serde::{Deserialize, Serialize};

    pub type SerializationError = bincode::ErrorKind;

    #[derive(Deserialize, Serialize, Debug, Copy, Clone)]
    pub struct EntityCastingSpell {
        pub entity: u32,
        pub timer: u64,
        pub max_timer: u64,
        pub spell_id: usize,
    }

    #[derive(Deserialize, Serialize, Debug, Copy, Clone)]
    pub struct EntitySpellCaster(pub u32);

    #[derive(Deserialize, Serialize, Debug, Copy, Clone)]
    pub struct EntityHealth {
        pub entity: u32,
        pub health: i64,
    }

    #[derive(Deserialize, Serialize, Debug, Copy, Clone)]
    pub struct EntityAura {
        pub entity: u32,
        pub aura_id: usize,
        pub remaining: u64,
    }

    #[derive(Deserialize, Serialize, Debug, Default)]
    pub struct WorldState {
        pub health: Vec<EntityHealth>,
        pub casting_spell: Vec<EntityCastingSpell>,
        pub spell_casters: Vec<EntitySpellCaster>,
        pub auras: Vec<EntityAura>,
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
    }
}
