use std::time::Duration;

use bevy::ecs::system::Resource;

pub(super) struct AuraData {
    pub name: String,
    pub duration: Duration,
    pub base_hp: Option<i64>,
}

impl AuraData {
    fn new(name: String, duration_ms: u64) -> AuraData {
        AuraData {
            name,
            duration: Duration::from_millis(duration_ms),
            base_hp: None,
        }
    }

    fn with_hp(mut self, base_hp: i64) -> Self {
        self.base_hp = Some(base_hp);
        self
    }
}

#[derive(Resource)]
pub(super) struct AuraList(pub Vec<AuraData>);

impl AuraList {
    pub(super) fn get_aura_data(&self, id: usize) -> Option<&AuraData> {
        self.0.get(id)
    }
}

pub(super) fn get_aura_list_resource() -> AuraList {
    AuraList(vec![
        AuraData::new("Immolated".into(), 2000).with_hp(-2),
        AuraData::new("Rotting".into(), 5000).with_hp(-4),
    ])
}
