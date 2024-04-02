use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::{
        entity::Entity,
        event::{Event, EventWriter, Events},
        system::{In, IntoSystem, Query, ResMut},
    },
    hierarchy::Children,
    utils::hashbrown::HashMap,
};

use crate::game::auras::shield::ShieldDamageEvent;

use super::{
    auras::{self, shield, AuraID},
    health::{self, HealthTickEvent},
};

/// Queue an effect onto the target.
#[derive(Event, Debug, Copy, Clone)]
pub struct EffectQueueEvent {
    pub target: Entity,
    pub health_effect: Option<i64>,
    pub aura_effect: Option<AuraID>,
}

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

#[derive(Clone, Copy)]
struct AbsorbDamage {
    total: Option<i64>,
    remaining: i64,
}

fn get_absorb_value(
    q_children: &Query<&Children>,
    q_shields: &Query<&auras::shield::StatusShield>,
    entity: Entity,
) -> Option<i64> {
    q_children
        .get(entity)
        .iter()
        .flat_map(|&e| q_shields.iter_many(e))
        .map(|f| f.value)
        .reduce(|f, v| f + v)
}

fn process_events(mut effect_queue: ResMut<Events<EffectQueueEvent>>) -> Vec<EffectPass> {
    effect_queue.drain().map(|v| v.into()).collect()
}

fn dispatch_shields(
    In(mut pass): In<Vec<EffectPass>>,
    q_children: Query<&Children>,
    q_shields: Query<&auras::shield::StatusShield>,
    mut shield_dmg_event_w: EventWriter<shield::ShieldDamageEvent>,
) -> Vec<EffectPass> {
    let mut absorb_cache: HashMap<Entity, AbsorbDamage> = HashMap::new();

    for effect in pass.iter_mut() {
        // only act if we're damaging our target
        if effect.health_effect.is_some_and(|v| v.is_negative()) {
            // get target absorb from cache or calculate from entity
            let target_absorb = match absorb_cache.get(&effect.target) {
                Some(&absorb) => absorb,
                None => {
                    let target_ab = get_absorb_value(&q_children, &q_shields, effect.target);
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
        shield_dmg_event_w.send(ShieldDamageEvent {
            damage: absorb_cache.total.unwrap_or(0) - absorb_cache.remaining,
            entity: entity,
        });
    }
    pass
}

fn dispatch_damage(
    In(second_pass): In<Vec<EffectPass>>,
    mut health_event_w: EventWriter<health::HealthTickEvent>,
    mut aura_add_event_w: EventWriter<auras::AddAuraEvent>,
) {
    for pass in second_pass {
        if let Some(hp) = pass.health_effect {
            health_event_w.send(HealthTickEvent {
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

pub struct EffectQueuePlugin;

impl Plugin for EffectQueuePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<Events<EffectQueueEvent>>();
        app.add_systems(
            FixedUpdate,
            process_events.pipe(dispatch_shields.pipe(dispatch_damage)),
        );
    }
}

#[cfg(test)]
mod test {

    use bevy::{
        app::{App, Update},
        ecs::{entity::Entity, event::Events, system::IntoSystem, world::World},
        hierarchy::BuildWorldChildren,
        utils,
    };

    use crate::game::{auras::shield, health};

    use super::{process_events, EffectQueueEvent};

    fn spawn_guy(world: &mut World) -> Entity {
        world.spawn(health::Health { hp: 50 }).id()
    }

    fn spawn_shield(world: &mut World, parent: Entity, val: i64) {
        let shield = world.spawn(shield::StatusShield { value: val }).id();
        world.entity_mut(parent).add_child(shield);
    }

    struct Test {
        events: Vec<EffectQueueEvent>,
        shields: Vec<i64>,
    }

    #[test]
    fn test_process_first() {
        let mut app = App::new();
        app.add_event::<EffectQueueEvent>();
        app.add_systems(Update, process_events.map(utils::dbg));
        let guy = spawn_guy(&mut app.world);
        app.update();

        let tests = vec![Test {
            shields: vec![30, 50, 100],
            events: vec![
                EffectQueueEvent {
                    health_effect: Some(-5),
                    target: guy,
                    aura_effect: None,
                },
                EffectQueueEvent {
                    health_effect: Some(-150),
                    target: guy,
                    aura_effect: None,
                },
                EffectQueueEvent {
                    health_effect: Some(-180),
                    target: guy,
                    aura_effect: None,
                },
            ],
        }];

        for test in tests {
            for shield in test.shields {
                spawn_shield(&mut app.world, guy, shield);
            }

            for ev in test.events {
                app.world
                    .get_resource_mut::<Events<EffectQueueEvent>>()
                    .unwrap()
                    .send(ev);
            }
            app.update();
        }
    }
}
