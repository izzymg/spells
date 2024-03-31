use bevy::{
    ecs::{
        event::EventWriter,
        system::{Query, Res},
    },
    hierarchy::Parent,
    log::debug,
};

use crate::health::{self, HealthTickEvent};

use super::{aura_types, resource, Aura};

// process ticking hp auras
pub(super) fn ticking_hp_system(
    aura_list: Res<resource::AuraList>,
    mut ev_w: EventWriter<health::HealthTickEvent>,
    query: Query<(&Parent, &Aura<{ aura_types::TICKING_HP }>)>,
) {
    for (parent, aura) in query.iter() {
        if aura.ticker.finished() {
            let aura_data = aura_list.get_aura_data(aura.aura_data_id).unwrap();
            ev_w.send(HealthTickEvent {
                entity: parent.get(),
                hp: aura_data.hp_per_tick.unwrap_or_default(),
            });

            debug!(
                "{} (id: {}) hp tick on {:?} ({}/{}s)",
                aura_data.name,
                aura.aura_data_id,
                parent,
                aura.timer.elapsed_secs(),
                aura_data.duration.as_secs()
            );
        }
    }
}
