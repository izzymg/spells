use bevy::{ecs::{component::Component, entity::Entity, system::{Commands, Query}}, hierarchy::DespawnRecursiveExt, log::debug};

/// Entity that can die
#[derive(Component)]
pub struct Health(pub i64);

/// Represents a change in an entities health this tick 
#[derive(Component)]
pub struct HealthTickSingle(pub i64);

/// Kills entities with no health
pub fn death_system(mut commands: Commands, query: Query<(Entity, &Health)>) {
    for (entity, health) in query.iter() {
        if health.0 <= 0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Ticks all HealthTick entities
pub fn health_tick_system(mut query: Query<(&mut Health, &HealthTickSingle)>) {
    for (mut health, tick) in query.iter_mut() {
        health.0 += tick.0;
    }
}

pub fn debug_health_tick_system(query: Query<(Entity, &Health, &HealthTickSingle)>) {
    for (entity, health, tick) in query.iter() {
        debug!("E{} tick: {} hp: {}", entity.index(), tick.0, health.0);
    }
}