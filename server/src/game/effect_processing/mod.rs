use std::time::Instant;

use bevy::{ecs::system::SystemParam, log, prelude::*};
use lib_spells::serialization;

use crate::game::{effect_application, events};

use super::ServerSets;

#[derive(SystemParam)]
struct ShieldQuery<'w, 's> {
    query_children: Query<'w, 's, &'static Children>,
    query_shields: Query<'w, 's, &'static mut effect_application::ShieldAura>,
}

impl<'w, 's> ShieldQuery<'w, 's> {
    /// Returns the current absorb value of a given entity based off its auras.
    fn get_total_entity_shielding(&self, entity: Entity) -> Option<i64> {
        self.query_children
            .get(entity)
            .iter()
            .flat_map(|&e| self.query_shields.iter_many(e))
            .map(|s| s.0)
            .reduce(|a, b| a + b)
    }

    /// Apply damage to shields on the entity. Damage should be positive (e.g. +400 to do 400 damage).
    fn apply_shield_damage(&mut self, entity: Entity, damage: i64) {
        let mut damage_left = damage;
        if let Ok(children) = self.query_children.get(entity) {
            // apply n damage to shields
            let mut iter = self.query_shields.iter_many_mut(children);
            while let Some(mut shield) = iter.fetch_next() {
                let applied_dmg = shield.0.min(damage_left);
                shield.0 -= applied_dmg;
                damage_left -= applied_dmg;
            }
        }
    }
}

#[derive(SystemParam)]
struct HealthQuery<'w, 's> {
    query_health: Query<'w, 's, &'static mut serialization::Health>,
}

impl<'w, 's> HealthQuery<'w, 's> {
    /// Update entity hp by effect. Negative to deal damage.
    fn apply_entity_hp(&mut self, entity: Entity, effect: i64) {
        if let Ok(mut hp) = self.query_health.get_mut(entity) {
            hp.0 += effect;
        }
    }
}

fn sys_bench_start(mut timing: ResMut<TimingResource>) {
    timing.processing_time = Instant::now();
}

fn sys_bench_fin(timing: Res<TimingResource>) {
    log::info!(
        "bench processing end: took {}ms ({}us)",
        timing.processing_time.elapsed().as_millis(),
        timing.processing_time.elapsed().as_micros()
    );
}

/// Apply damage events with respect to active target auras.
fn sys_process_damage_effects(
    effect_events: Res<Events<events::EffectQueueEvent>>,
    mut shield_query: ShieldQuery,
    mut health_query: HealthQuery,
) {
    for effect in effect_events.get_reader().read(&effect_events) {
        if effect.health_effect.is_none() {
            continue;
        }
        let mut health_effect = effect.health_effect.unwrap();

        let is_damaging = effect.health_effect.is_some_and(|f| f.is_negative());
        let target_shielding = shield_query.get_total_entity_shielding(effect.target);
        if is_damaging && target_shielding.is_some() {
            let shield_damage = health_effect.abs().min(target_shielding.unwrap()); // damage to shields <= total shielding
            log::debug!("{:?} absorbs {} damage", effect.target, shield_damage);
            health_effect = (target_shielding.unwrap() + health_effect).min(0);
            shield_query.apply_shield_damage(effect.target, shield_damage);
        }
        health_query.apply_entity_hp(effect.target, health_effect);
        log::debug!("{:?} {:+} hp", effect.target, health_effect);
    }
}

/// Process aura events
fn sys_process_aura_effects(
    effect_events: Res<Events<events::EffectQueueEvent>>,
    mut ev_w: EventWriter<events::AddAuraEvent>,
) {
    for effect in effect_events.get_reader().read(&effect_events) {
        if let Some(aura) = effect.aura_effect {
            ev_w.send(events::AddAuraEvent {
                aura_id: aura,
                target_entity: effect.target,
            });
        }
    }
}

fn sys_drain_effect_evs(mut effect_events: ResMut<Events<events::EffectQueueEvent>>) {
    effect_events.clear();
}

#[derive(Resource)]
struct TimingResource {
    processing_time: Instant,
}

pub struct EffectPlugin;
impl Plugin for EffectPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(TimingResource {
            processing_time: Instant::now(),
        });
        app.add_systems(
            FixedUpdate,
            (
                sys_bench_start,
                sys_process_damage_effects,
                sys_process_aura_effects,
                sys_drain_effect_evs,
                sys_bench_fin,
            )
                .chain()
                .in_set(ServerSets::EffectProcessing),
        );
    }
}

#[cfg(test)]
mod tests {

    use bevy::{
        app::{self, Update},
        ecs::event::Events,
        hierarchy::BuildWorldChildren,
    };
    use lib_spells::serialization;

    use crate::game::{effect_application, events};

    use super::sys_process_damage_effects;

    #[test]
    fn test_shielded_damage() {
        let hp = 30;
        let shields = vec![10, 5, 3];
        let total_shielding = shields.clone().into_iter().reduce(|a, b| a + b).unwrap();
        let hits = vec![-18, -2, -4];
        let total_damage = hits.clone().into_iter().reduce(|a, b| a + b).unwrap();
        let expect_hp = hp + (total_shielding + total_damage);

        let mut app = app::App::new();
        app.init_resource::<Events<events::EffectQueueEvent>>();
        app.add_systems(Update, sys_process_damage_effects);

        let skele = app.world.spawn(serialization::Health(hp)).id();
        for shield in shields {
            let child = app.world.spawn(effect_application::ShieldAura(shield)).id();
            app.world.entity_mut(skele).add_child(child);
        }

        for hit in hits {
            app.world
                .get_resource_mut::<Events<events::EffectQueueEvent>>()
                .unwrap()
                .send(events::EffectQueueEvent {
                    aura_effect: None,
                    health_effect: Some(hit),
                    target: skele,
                });
        }
        app.update();

        let remaining_hp = app.world.get::<serialization::Health>(skele).unwrap().0;
        assert_eq!(remaining_hp, expect_hp);
    }
}
