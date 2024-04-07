use crate::game::effects;
use bevy::{
    ecs::{
        event::EventWriter,
        schedule::IntoSystemConfigs,
        system::{Query, Res},
    },
    hierarchy::Parent,
    log,
    time::Time,
};

use super::TickingEffectAura;

// Process ticking effects
fn sys_ticking_effect_aura(
    mut query: Query<(&Parent, &super::Aura, &mut TickingEffectAura)>,
    aura_resource: super::resource::AuraSysResource,
    time: Res<Time>,
    mut ev_w: EventWriter<effects::EffectQueueEvent>,
) {
    for (parent, effect, mut hp_tick) in query.iter_mut() {
        hp_tick.ticker.tick(time.delta());
        if hp_tick.ticker.just_finished() {
            if let Some(effect_data) = aura_resource.get_status_effect_data(effect.id) {
                ev_w.send(effects::EffectQueueEvent {
                    target: parent.get(),
                    health_effect: Some(effect_data.base_multiplier),
                    aura_effect: None,
                });
            } else {
                log::error!("no status effect at id: {}", effect.id)
            }
        }
    }
}

pub fn get_configs() -> impl IntoSystemConfigs<()> {
    sys_ticking_effect_aura.into_configs()
}
