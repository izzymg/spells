use bevy::{
    ecs::{event::EventReader, schedule::IntoSystemConfigs, system::Query},
    log,
};

use super::{Health, HealthTickEvent};

/// Damage entities in health tick events
fn health_tick_system(
    mut ev_health_tick: EventReader<HealthTickEvent>,
    mut q_health: Query<&mut Health>,
) {
    for ev in ev_health_tick.read() {
        if let Ok(mut health) = q_health.get_mut(ev.entity) {
            health.hp += ev.hp;
            log::debug!("{:?} ticked for {} hp: {}", ev.entity, ev.hp, health.hp);
        }
    }
}

pub fn get_config() -> impl IntoSystemConfigs<()> {
    health_tick_system.into_configs()
}
