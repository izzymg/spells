use bevy::{
    ecs::{
        event::EventWriter, system::{Query, Res}
    }, hierarchy::Parent
};

use crate::health::{self, HealthTickEvent};

use super::{resource, Aura, aura_types};

// process ticking damage auras
pub(super) fn ticking_damage_system(
    aura_list: Res<resource::AuraList>,
    mut ev_w: EventWriter<health::HealthTickEvent>,
    query: Query<(&Parent, &Aura<{aura_types::TICKING_HP}>)>
) {
    for (parent, burning_aura) in query.iter() {
        let aura_data = aura_list.get_aura_data(burning_aura.aura_data_id).unwrap();
        ev_w.send(HealthTickEvent {
            entity: parent.get(),
            hp: aura_data.base_hp.unwrap_or(0),
        });
    }

}
