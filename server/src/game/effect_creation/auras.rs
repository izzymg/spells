// Aura systems that cause an effect event

use bevy::prelude::*;
use lib_spells::serialization;

use crate::game::{
    assets, effect_application,
    events::{self, EffectQueueEvent},
};

/// Tick each aura by delta.
pub(super) fn sys_tick_ticking_auras(
    mut ticking_query: Query<&mut effect_application::TickingEffectAura>,
    time: Res<Time>,
) {
    for mut ticking in ticking_query.iter_mut() {
        ticking.0.tick(time.delta());
    }
}

/// Apply auras that have ticked.
pub(super) fn sys_apply_aura_tick(
    auras_asset: Res<assets::AurasAsset>,
    mut effect_ev_w: ResMut<Events<events::EffectQueueEvent>>,
    ticking_query: Query<(
        &Parent,
        &serialization::Aura,
        &effect_application::TickingEffectAura,
    )>,
) {
    for (parent, aura, _) in ticking_query.iter().filter(|t| t.2 .0.finished()) {
        if let Some(aura_data) = auras_asset.lookup(aura.id) {
            effect_ev_w.send(EffectQueueEvent {
                aura_effect: None,
                health_effect: Some(aura_data.base_multiplier),
                target: parent.get(),
            });
        }
    }
}
