/// status effect that ticks HP change onto the target

use std::time::Duration;

use bevy::{
    ecs::{
        component::Component, event::EventWriter, system::{Query, Res}
    },
    hierarchy::Parent,
    log,
    time::{Time, Timer, TimerMode},
};

use crate::game::health::HealthTickEvent;

const TICK_RATE: Duration = Duration::from_millis(1000);

#[derive(Component)]
pub(super) struct StatusTickingHP {
    ticker: Timer,
}

impl StatusTickingHP {
    pub(super) fn new() -> StatusTickingHP {
        StatusTickingHP {
            ticker: Timer::new(TICK_RATE, TimerMode::Repeating)
        }
    }
}

// process ticking damage
pub(super) fn ticking_damage_system(
    mut query: Query<(&Parent, &super::StatusEffect, &mut StatusTickingHP)>,
    status_system: super::resource::StatusEffectSystem,
    time: Res<Time>,
    mut ev_w: EventWriter<HealthTickEvent>,
) {
    for (parent, effect, mut hp_tick) in query.iter_mut() {
        hp_tick.ticker.tick(time.delta());
        if hp_tick.ticker.just_finished() {
            if let Some(effect_data) = status_system.get_status_effect_data(effect.id) {

                ev_w.send(HealthTickEvent {
                    entity: parent.get(),
                    hp: effect_data.base_multiplier,
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
