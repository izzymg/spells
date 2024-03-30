use bevy::{ecs::{component::Component, entity::Entity, event::{Event, EventReader}, system::{Commands, Query}}, hierarchy::DespawnRecursiveExt, log::debug};

/// Entity that can die
#[derive(Debug, Component)]
pub struct Health(pub i64);

#[derive(Debug, Event)]
pub struct HealthTickEvent{
    pub entity: Entity,
    pub hp: i64,
}


/// Kills entities with no health
pub fn death_system(mut commands: Commands, query: Query<(Entity, &Health)>) {
    for (entity, health) in query.iter() {
        if health.0 <= 0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Ticks all HealthTick entities
pub fn health_tick_system(
    mut ev_r: EventReader<HealthTickEvent>,
    mut query: Query<&mut Health>
) {
        for ev in ev_r.read() {
            if let Ok(mut health) = query.get_mut(ev.entity) {
                health.0 += ev.hp;
                debug!("E{} tick: {} hp: {}", ev.entity.index(), ev.hp, health.0);
            }
        }
}