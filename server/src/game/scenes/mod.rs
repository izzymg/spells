use crate::game::{components, events};
use bevy::{log, prelude::*};

pub fn sys_many_effects(world: &mut World) {
    let n_defenders = 1;
    let defender_hp = 55;
    let n_shields = 1;
    let shield_val = 20;
    let n_effects_per_defender = 4;
    let effect_dmg = -3;

    log::info!(
        "processing {} effects",
        n_defenders * n_effects_per_defender
    );

    let mut defender_entities = vec![];
    for _ in 0..n_defenders {
        let defender = world.spawn(components::Health { hp: defender_hp }).id();
        defender_entities.push(defender);
        for _ in 0..n_shields {
            let shield = world
                .spawn(components::ShieldAura { value: shield_val })
                .id();
            world.entity_mut(defender).add_child(shield);
        }
    }
    for target in defender_entities.iter() {
        for _ in 0..n_effects_per_defender {
            world
                .get_resource_mut::<Events<events::EffectQueueEvent>>()
                .unwrap()
                .send(events::EffectQueueEvent {
                    target: *target,
                    health_effect: Some(effect_dmg),
                    aura_effect: None,
                });
        }
    }
}
