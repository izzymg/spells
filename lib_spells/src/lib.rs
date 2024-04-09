pub const CLIENT_EXPECT: &str = "SPELLCLIENT OK 0.1\n";
pub const SERVER_HEADER: &str = "SPELLSERVER 0.1\n";

pub mod serialization {
    use bincode;
    use serde::{Deserialize, Serialize};

    pub type SerializationError = bincode::ErrorKind;

    #[derive(Deserialize, Serialize, Debug, Copy, Clone)]
    pub struct EntityCaster {
        pub entity: u32,
        pub timer: u128,
        pub max_timer: u128,
        pub spell_id: usize,
    }

    #[derive(Deserialize, Serialize, Debug, Copy, Clone)]
    pub struct EntityHealth {
        pub entity: u32,
        pub health: i64,
    }

    #[derive(Deserialize, Serialize, Debug, Copy, Clone)]
    pub struct EntityAura {
        pub entity: u32,
        pub aura_id: usize,
        pub remaining: u128,
    }

    #[derive(Deserialize, Serialize, Debug, Default)]
    pub struct WorldState {
        pub health: Vec<EntityHealth>,
        pub casters: Vec<EntityCaster>,
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
