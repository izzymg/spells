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
    log,
};


pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<HealthTickEvent>()
            .add_systems(FixedUpdate, death_system.before(health_tick_system))
            .add_systems(FixedUpdate, health_tick_system);
    }
}

/// Entity that can die
#[derive(Debug, Component, Default)]
pub struct Health {
    pub hp: i64,
}

impl Health {
    pub fn new(hp: i64) -> Health {
        Health { hp }
    }
}

#[derive(Debug, Event)]
pub struct HealthTickEvent {
    pub entity: Entity,
    pub hp: i64,
}

/// Kills entities with no health
fn death_system(mut commands: Commands, query: Query<(Entity, &Health)>) {
    for (entity, health) in query.iter() {
        if health.hp <= 0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Ticks all HealthTick entities
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
