use bevy::{
    ecs::{
        entity::Entity,
        event::{EventWriter, Events},
        schedule::IntoSystemConfigs,
        system::{In, IntoSystem, Query, ResMut},
    },
    hierarchy::Children,
    utils::hashbrown::HashMap,
};

use crate::game::{health};

use super::{
    auras::{self, AuraID},
    EffectQueueEvent,
};

/// Pass of event process & simulation.
#[derive(Debug, Copy, Clone)]
struct EffectPass {
    target: Entity,
    health_effect: Option<i64>,
    aura_effect: Option<AuraID>,
}

impl From<EffectQueueEvent> for EffectPass {
    fn from(value: EffectQueueEvent) -> Self {
        Self {
            health_effect: value.health_effect,
            target: value.target,
            aura_effect: value.aura_effect,
        }
    }
}
/// Tracking simulated absorb shield effects
#[derive(Clone, Copy)]
struct AbsorbDamage {
    total: Option<i64>,
    remaining: i64,
}

/// Returns the current absorb value of a given entity based off its auras.
fn get_total_entity_shielding(
    q_children: &Query<&Children>,
    q_shields: &Query<&auras::ShieldAura>,
    entity: Entity,
) -> Option<i64> {
    q_children
        .get(entity)
        .iter()
        .flat_map(|&e| q_shields.iter_many(e))
        .map(|f| f.value)
        .reduce(|f, v| f + v)
}

/// Maps all effect queue events into our pass pipe.
fn sys_process_events(mut effect_queue: ResMut<Events<EffectQueueEvent>>) -> Vec<EffectPass> {
    effect_queue.drain().map(|v| v.into()).collect()
}

/// Simulate damage to enemy shields then dispatch low level events, returning "leftover" damage to our next pass.
fn sys_dispatch_shields(
    In(mut pass): In<Vec<EffectPass>>,
    q_children: Query<&Children>,
    q_shields: Query<&auras::ShieldAura>,
    mut shield_dmg_event_w: EventWriter<auras::ShieldDamageEvent>,
) -> Vec<EffectPass> {
    let mut absorb_cache: HashMap<Entity, AbsorbDamage> = HashMap::new();

    for effect in pass.iter_mut() {
        // only act if we're damaging our target
        if effect.health_effect.is_some_and(|v| v.is_negative()) {
            // get target absorb from cache or calculate from entity
            let target_absorb = match absorb_cache.get(&effect.target) {
                Some(&absorb) => absorb,
                None => {
                    let target_ab =
                        get_total_entity_shielding(&q_children, &q_shields, effect.target);
                    let ab = AbsorbDamage {
                        total: target_ab,
                        remaining: target_ab.unwrap_or_default(),
                    };

                    absorb_cache.insert(effect.target, ab);
                    ab
                }
            };

            // ensure we only do work if our target entity has a shield > 0
            if target_absorb.total.is_some_and(|v| v > 0) {
                // update remaining in cache
                let remaining = target_absorb.remaining + effect.health_effect.unwrap();
                absorb_cache.insert(
                    effect.target,
                    AbsorbDamage {
                        remaining: remaining.max(0),
                        ..target_absorb
                    },
                );

                // apply "spillover" damage
                if remaining < 0 {
                    effect.health_effect = Some(remaining);
                } else {
                    effect.health_effect = None;
                }
            }
        }
    }

    // send all our damage events
    for (entity, absorb_cache) in absorb_cache {
        shield_dmg_event_w.send(auras::ShieldDamageEvent {
            damage: absorb_cache.total.unwrap_or(0) - absorb_cache.remaining,
            entity: entity,
        });
    }
    pass
}

/// Dispatch low level damage events.
fn sys_dispatch_damage(
    In(second_pass): In<Vec<EffectPass>>,
    mut health_event_w: EventWriter<health::HealthTickEvent>,
    mut aura_add_event_w: EventWriter<auras::AddAuraEvent>,
) {
    for pass in second_pass {
        if let Some(hp) = pass.health_effect {
            health_event_w.send(health::HealthTickEvent {
                entity: pass.target,
                hp,
            });
        }

        if let Some(aura_id) = pass.aura_effect {
            aura_add_event_w.send(auras::AddAuraEvent {
                aura_id,
                target_entity: pass.target,
            });
        }
    }
}

pub fn get_configs() -> impl IntoSystemConfigs<()> {
    sys_process_events
        .pipe(sys_dispatch_shields.pipe(sys_dispatch_damage))
        .into_configs()
}
