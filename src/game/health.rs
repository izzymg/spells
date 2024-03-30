use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader},
        schedule::IntoSystemConfigs,
        system::{Commands, Query},
    },
    hierarchy::DespawnRecursiveExt,
    log::debug,
};

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
        .add_event::<HealthTickEvent>()
        .add_systems(FixedUpdate, death_system.before(health_tick_system))
        .add_systems(FixedUpdate, health_tick_system);
    }
}

/// Entity that can die
#[derive(Debug, Component)]
pub struct Health(pub i64);

#[derive(Debug, Event)]
pub struct HealthTickEvent {
    pub entity: Entity,
    pub hp: i64,
}

/// Kills entities with no health
fn death_system(mut commands: Commands, query: Query<(Entity, &Health)>) {
    for (entity, health) in query.iter() {
        if health.0 <= 0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Ticks all HealthTick entities
fn health_tick_system(mut ev_r: EventReader<HealthTickEvent>, mut query: Query<&mut Health>) {
    for ev in ev_r.read() {
        if let Ok(mut health) = query.get_mut(ev.entity) {
            health.0 += ev.hp;
            debug!("E{} tick: {} hp: {}", ev.entity.index(), ev.hp, health.0);
        }
    }
}