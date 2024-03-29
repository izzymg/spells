use bevy::{ecs::{component::Component, entity::Entity, system::{Commands, Query}}, hierarchy::DespawnRecursiveExt};

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
            println!("killed E{}", entity.index());
        }
    }
}

/// Ticks all HealthTick entities
pub fn health_tick_system(mut query: Query<(Entity, &mut Health, &HealthTickSingle)>) {
    for (entity, mut health, tick) in query.iter_mut() {
        health.0 += tick.0;
        println!("E{} hp: {}", entity.index(), health.0);
        
    }
}