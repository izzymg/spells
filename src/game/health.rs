use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        query::{Has, With},
        schedule::IntoSystemConfigs,
        system::{Commands, Query},
    },
    hierarchy::{Children, DespawnRecursiveExt},
    log::{self, debug},
};

use super::status_effect::{self, shield::ShieldDamageEvent};

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
    pub fn process_tick(&mut self, hp_change: i64) {
        self.hp += hp_change;
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
    q_children: Query<&Children>,
    q_shields: Query<&status_effect::shield::StatusShield>,
    mut shield_damage_ev_w: EventWriter<status_effect::shield::ShieldDamageEvent>,
) {
    for ev in ev_health_tick.read() {
        if let Ok(mut health) = q_health.get_mut(ev.entity) {
            let mut damage = ev.hp;
            if ev.hp.is_negative() {
                // get total value of hit entity's shields
                if let Some(shield_total) = q_children
                    .get(ev.entity)
                    .iter()
                    .flat_map(|&e| q_shields.iter_many(e))
                    .map(|f| f.value)
                    .reduce(|f, v| f + v)
                {
                    // notify of shield damage
                    shield_damage_ev_w.send(ShieldDamageEvent {
                        damage: ev.hp.abs(),
                        entity: ev.entity,
                    });
                    // apply post-shielded damage
                    damage = (shield_total + ev.hp).min(0);
                }
            }

            health.hp += damage;
            log::debug!("{:?} ticked for {}, hp: {}", ev.entity, damage, health.hp);
        }
    }
}
