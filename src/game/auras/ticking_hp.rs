/// status effect that ticks HP change onto the target

use std::{default, time::Duration};

use bevy::{
    ecs::{
        component::Component, event::EventWriter, system::{Query, Res}
    },
    hierarchy::Parent,
    log,
    time::{Time, Timer, TimerMode},
};

use crate::effect_application;

const TICK_RATE: Duration = Duration::from_millis(1000);

#[derive(Component)]
pub(super) struct AuraTickingHealth {
    ticker: Timer,
}

impl AuraTickingHealth {
    pub(super) fn new() -> AuraTickingHealth {
        AuraTickingHealth {
            ticker: Timer::new(TICK_RATE, TimerMode::Repeating)
        }
    }
}

// process ticking damage
pub(super) fn ticking_damage_system(
    mut query: Query<(&Parent, &super::Aura, &mut AuraTickingHealth)>,
    aura_resource: super::resource::AuraSysResource,
    time: Res<Time>,
    mut ev_w: EventWriter<effect_application::EffectQueueEvent>,
) {
    for (parent, effect, mut hp_tick) in query.iter_mut() {
        hp_tick.ticker.tick(time.delta());
        if hp_tick.ticker.just_finished() {
            if let Some(effect_data) = aura_resource.get_status_effect_data(effect.id) {

                ev_w.send(effect_application::EffectQueueEvent {
                    target: parent.get(),
                    health_effect: Some(effect_data.base_multiplier),
                    aura_effect: None,
                });
                log::debug!(
                    "{:?} ticks {} ({}/{}s)",
                    parent.get(),
                    effect_data.name,
                    effect.get_remaining_time(effect_data.duration).as_secs(),
                    effect_data.duration.as_secs(),
                );
            } else {
                log::error!("no status effect at id: {}", effect.id)
            }
        }
    }
}
