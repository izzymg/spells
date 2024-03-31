use std::time::Duration;
use bevy::ecs::system::Resource;

use super::AURA_TICK_RATE;

pub(super) struct AuraData {
    pub name: String,
    pub duration: Duration,
    pub hp_per_tick: Option<i64>,
}

impl AuraData {
    fn new(name: String, duration_ms: u64) -> AuraData {
        AuraData {
            name,
            duration: Duration::from_millis(duration_ms),
            hp_per_tick: None,
        }
    }

    fn with_hps(mut self, per_second: i64) -> Self {
        self.hp_per_tick = Some((AURA_TICK_RATE.as_secs_f64() * (per_second as f64)) as i64);
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
        AuraData::new("Immolated".into(), 5000).with_hps(-1),
        AuraData::new("Rotting".into(), 2000).with_hps(-4),
    ])
}
